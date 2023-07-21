use clap::{Args, Parser, Subcommand};

mod image;
mod partitions;
mod util;


#[derive(Debug, Parser)]
#[command(name = "diskedit", version)]
#[command(bin_name = "diskedit")]
struct DiskEditCLI {
    #[command(subcommand)]
    command: DiskEditCommands,
}


#[derive(Debug, Subcommand)]
enum DiskEditCommands {
    Image(ImageArgs),
    Partitions(PartitionsArgs),
    Filesystem(FilesystemArgs),
}


#[derive(Debug, Args)]
struct ImageArgs {
    #[command(subcommand)]
    command: ImageCommands,
}


#[derive(Debug, Subcommand)]
enum ImageCommands {
    Create {
        path: String,
        size: String,
        #[arg(short, long)]
        partition: Vec<String>,
    }
}


#[derive(Debug, Args)]
struct PartitionsArgs {
    #[command(subcommand)]
    command: PartitionsCommands,
}


#[derive(Debug, Subcommand)]
enum PartitionsCommands {
    Read {
        device: String,
    },
    Write {
        device: String,
        #[arg(short, long)]
        partition: Vec<String>,
    },
}


#[derive(Debug, Args)]
struct FilesystemArgs {
    #[command(subcommand)]
    command: FilesystemCommands,
}


#[derive(Debug, Subcommand)]
enum FilesystemCommands {
    List {
        device: String,
        path: String,
    },
}


fn main() {
    let args = DiskEditCLI::parse();
    match args.command {
        DiskEditCommands::Image(image_args) => {
            match image_args.command {
                ImageCommands::Create { path, size, partition } => {
                    let partitions = partition.iter().map(|p| p.as_str())
                        .collect::<Vec<&str>>();
                    match image::create_image(&path, &size) {
                        Ok(()) => {
                            println!("Created image {} with size {}", path, size);
                            match partitions::write_partitions(&path, partitions) {
                                Ok(()) => (),
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                },
            }
        },
        DiskEditCommands::Partitions(partitions_args) => {
            match partitions_args.command {
                PartitionsCommands::Read { device } => {
                    match partitions::read_partitions(&device) {
                        Ok(()) => (),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                },
                PartitionsCommands::Write { device, partition } => {
                    let partitions = partition.iter().map(|p| p.as_str()).collect();
                    match partitions::write_partitions(&device, partitions) {
                        Ok(()) => (),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                },
            }
        },
        DiskEditCommands::Filesystem(filesystem_args) => {
            match filesystem_args.command {
                FilesystemCommands::List { device, path } => {
                    println!("List filesystem {} {}", device, path);
                },
            }
        },
    }
}
