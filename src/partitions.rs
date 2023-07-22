use gpt;
use uuid::Uuid;

use crate::util::{Offset, Result};


struct Partition {
    name: Option<String>,
    start: Offset,
    end: Offset,
    partition_type: gpt::partition_types::Type,
}

impl Partition {
    fn size(&self, relative_to: u64) -> u64 {
        let start = self.start.to_bytes(relative_to);
        let end = self.end.to_bytes(relative_to);
        end - start
    }
}


fn parse_partition_type(partition_type: &str) -> Result<gpt::partition_types::Type> {
    match gpt::partition_types::Type::from_name(partition_type) {
        Ok(t) => Ok(t),
        Err(_) => Err(crate::util::Error::InvalidPartitionType),
    }
}


fn parse_partition(partition: &str) -> Result<Partition> {
    let spec = partition.split(':').collect::<Vec<&str>>();
    let (name, start, end, partition_type) = match Offset::parse(spec[0]) {
        Ok(start) => {
            let end = Offset::parse(spec[1])?;
            let partition_type = if spec.len() == 3 {
                parse_partition_type(spec[2])?
            } else {
                gpt::partition_types::LINUX_FS
            };
            (None, start, end, partition_type)
        },
        Err(_) => {
            let name = spec[0].to_string();
            let start = Offset::parse(spec[1])?;
            let end = Offset::parse(spec[2])?;
            let partition_type = if spec.len() == 4 {
                parse_partition_type(spec[3])?
            } else {
                gpt::partition_types::LINUX_FS
            };
            (Some(name), start, end, partition_type)
        },
    };

    Ok(Partition {
        name,
        start,
        end,
        partition_type,
    })
}


pub fn read_partitions(device: &str) -> Result<()> {
    let devicepath = std::path::Path::new(device);
    let cfg = gpt::GptConfig::new().writable(false);
    let disk = cfg.open(devicepath)?;

    println!("Disk (primary) header: {:#?}", disk.primary_header());
    println!("Partition layout: {:#?}", disk.partitions());

    Ok(())
}


pub fn write_partitions(device: &str, partitions: Vec<&str>) -> Result<()> {
    let devicepath = std::path::Path::new(device);
    let mut devicefile = std::fs::OpenOptions::new()
        .read(true).write(true).open(devicepath)?;

    // Determine if device is a block device or a file
    let metadata = std::fs::metadata(devicepath)?;

    // Compute device size
    let mut device_size = metadata.len();
    if device_size == 0 {
        // Determine block device size from sysfs
        let sysfs_path = std::path::Path::new("/sys/block")
            .join(devicepath.file_name().unwrap());

        if !sysfs_path.exists() {
            return Err(crate::util::Error::InvalidDevice);
        }

        let sysfs_size = std::fs::read_to_string(sysfs_path.join("size"))?;
        let sysfs_size = sysfs_size.trim().parse::<u64>()?;
        device_size = sysfs_size * 512;
    };

    // Write protective MBR
    let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
        u32::try_from((device_size / 512) - 1).unwrap_or(0xFF_FF_FF_FF));
    mbr.overwrite_lba0(&mut devicefile)?;

    // Create GPT disk
    let mut disk = gpt::GptConfig::default()
        .initialized(false).writable(true)
        .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
        .create_from_device(Box::new(devicefile), None)?;
    let partitions = partitions.iter().map(|p|
        parse_partition(p)).collect::<Result<Vec<Partition>>>()?;

    // Initialize blank partitions
    disk.update_partitions(
        std::collections::BTreeMap::<u32, gpt::partition::Partition>::new())?;

    // Clear existing partitions
    let part_guids = disk.partitions().iter().map(|(&n, p)| (n, p.part_guid))
        .collect::<Vec<(u32, Uuid)>>();
    for (part_num, part_guid) in part_guids {
        disk.remove_partition(Some(part_num), Some(part_guid))?;
    }

    // Create new partitions
    for partition in partitions {
        let partition_size = partition.size(device_size);
        let partition_type = partition.partition_type;
        let partition_name = &partition.name.unwrap_or("".to_string());
        disk.add_partition(
            partition_name,
            partition_size,
            partition_type,
            0,
            None,
        )?;
    }

    // Write new partition table
    println!("Writing new partition table to {}", device);
    disk.write()?;

    Ok(())
}
