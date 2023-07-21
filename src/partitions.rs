use gpt;
use uuid::Uuid;

use crate::util::Result;


#[derive(Clone, Copy, Debug)]
enum Offset {
    Bytes(u64),
    Kilobytes(u64),
    Megabytes(u64),
    Gigabytes(u64),
    Percent(u8),
}


struct Partition {
    name: Option<String>,
    start: Offset,
    end: Offset,
}

impl Partition {
    fn size(&self, relative_to: u64) -> u64 {
        let start = match self.start {
            Offset::Bytes(start) => start,
            Offset::Kilobytes(start) => start * 1024,
            Offset::Megabytes(start) => start * 1024 * 1024,
            Offset::Gigabytes(start) => start * 1024 * 1024 * 1024,
            Offset::Percent(start) => {
                (relative_to as f64 * (start as f64 / 100.0)) as u64
            },
        };

        let end = match self.end {
            Offset::Bytes(end) => end,
            Offset::Kilobytes(end) => end * 1024,
            Offset::Megabytes(end) => end * 1024 * 1024,
            Offset::Gigabytes(end) => end * 1024 * 1024 * 1024,
            Offset::Percent(end) => {
                (relative_to as f64 * (end as f64 / 100.0)) as u64
            },
        };

        end - start
    }
}

fn parse_partition(partition: &str) -> Result<Partition> {
    let spec = partition.split(':').collect::<Vec<&str>>();

    if spec.len() < 2 || spec.len() > 3 {
        return Err(crate::util::Error::InvalidPartitionSpecification);
    }

    let name = if spec.len() == 3 {
        Some(spec[0].to_string())
    } else {
        None
    };

    let start = parse_partition_offset(spec[spec.len() - 2])?;
    let end = parse_partition_offset(spec[spec.len() - 1])?;

    Ok(Partition {
        name,
        start,
        end,
    })
}


fn parse_partition_offset(offset: &str) -> Result<Offset> {
    let offset = offset.trim();

    if offset.ends_with('%') {
        let percent = offset.trim_end_matches('%').parse::<u8>()?;
        if percent > 100 {
            return Err(crate::util::Error::InvalidPartitionSpecification);
        }
        return Ok(Offset::Percent(percent));
    } else if offset.ends_with('K') {
        let kilobytes = offset.trim_end_matches('K').parse::<u64>()?;
        return Ok(Offset::Kilobytes(kilobytes));
    } else if offset.ends_with('M') {
        let megabytes = offset.trim_end_matches('M').parse::<u64>()?;
        return Ok(Offset::Megabytes(megabytes));
    } else if offset.ends_with('G') {
        let gigabytes = offset.trim_end_matches('G').parse::<u64>()?;
        return Ok(Offset::Gigabytes(gigabytes));
    } else if offset.ends_with('B') {
        let bytes = offset.trim_end_matches('B').parse::<u64>()?;
        return Ok(Offset::Bytes(bytes));
    } else {
        let bytes = offset.parse::<u64>()?;
        return Ok(Offset::Bytes(bytes));
    }
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
        let partition_name = &partition.name.unwrap_or("".to_string());
        disk.add_partition(
            partition_name,
            partition_size,
            gpt::partition_types::LINUX_FS,
            0,
            None,
        )?;
    }

    // Write new partition table
    println!("Writing new partition table to {}", device);
    disk.write()?;

    Ok(())
}
