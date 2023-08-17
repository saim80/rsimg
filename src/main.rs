// Project: rsimg

use clap::Parser;
use image::GenericImageView;
use walkdir::WalkDir;

#[derive(Parser, Default)]
struct CLI {
    #[clap(short = 's', long = "source", default_value = ".")]
    source_path: std::path::PathBuf,
    #[clap(short = 't', long = "task", default_value = "resize")]
    task: String,
    #[clap(short = 'o', long = "options", default_value = "size=128x128")]
    options: String,
}

#[derive(Copy, Clone)]
struct SizeArgs {
    width: u32,
    height: u32,
    scale: f32,
}

fn process_directory(source_path: std::path::PathBuf, size: SizeArgs, filter: image::imageops::FilterType, executor: fn(std::path::PathBuf, SizeArgs, image::imageops::FilterType)) {
    // the source path is a directory. iterate all children. for images, perform resize.
    // iterate all children.
    for entry in WalkDir::new(source_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            // check if path is an image.
            if let Some(extension) = path.extension() {
                if let Some(extension) = extension.to_str() {
                    if extension == "png" || extension == "jpg" || extension == "jpeg" {
                        executor(path.to_path_buf(), size, filter);
                    }
                }
            }
        }
    }
}

fn resize_by_scale(source_path: std::path::PathBuf, scale: f32, filter: image::imageops::FilterType) {
    let target_path = source_path.clone();
    // open image.
    let image = image::open(source_path).unwrap();
    // get image dimensions.
    let (width, height) = image.dimensions();
    // calculate new dimensions.
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;
    // resize image.
    let resized_image = image::imageops::resize(&image, new_width, new_height, filter);
    // save resized image.
    resized_image.save(target_path).unwrap();
}

fn resize_by_size(source_path: std::path::PathBuf, size: (u32, u32), filter: image::imageops::FilterType) {
    let target_path = source_path.clone();
    // open image.
    let image = image::open(source_path).unwrap();
    // resize image.
    let resized_image = image::imageops::resize(&image, size.0, size.1, filter);
    // save resized image.
    resized_image.save(target_path).unwrap();
}

/// Resize images in a directory.
///
///  # Supported Options
///
///  ## size (Required)
///  - {width}x{height}
///  - {percentage}%
///
///  ## filter
///  - nearest
///  - linear
///  - cubic (default)
///  - gaussian
///  - lanczos3
///
///  @param source_path Path to source directory.
///  @param options Options for resizing.
fn resize(source_path: std::path::PathBuf, options: std::collections::HashMap<&str, &str>) {
    // initialize imageops filter type.
    let mut filter = image::imageops::FilterType::CatmullRom;
    // check options dictionary if filter is specified.
    if options.contains_key("filter") {
        // get filter value.
        let filter_value = options.get("filter").unwrap();
        // convert filter value to string.
        let string_value = filter_value.to_string();

        // check if filter value is valid.
        match string_value.as_str() {
            "default" => filter = image::imageops::FilterType::CatmullRom,
            "nearest" => filter = image::imageops::FilterType::Nearest,
            "linear" => filter = image::imageops::FilterType::Triangle,
            "cubic" => filter = image::imageops::FilterType::CatmullRom,
            "gaussian" => filter = image::imageops::FilterType::Gaussian,
            "lanczos3" => filter = image::imageops::FilterType::Lanczos3,
            _ => filter = image::imageops::FilterType::CatmullRom,
        }
    }

    // check size option.
    if !options.contains_key("size") {
        panic!("Missing required option: size");
    }

    // get size value.
    let size_value = options.get("size").unwrap();
    // split size value by x.
    let size: Vec<&str> = size_value.split("x").collect();
    // check if size is valid.
    if size.len() != 2 {
        // check if size is specified in percentage.
        if size_value.ends_with("%")
        {
            // strip percentage sign.
            let size_value = size_value.strip_suffix("%").unwrap();
            // parse size value to float.
            let percentage: f32 = size_value.parse().unwrap();
            // turn percentage into scale (0.0 - 1.0)
            let scale = percentage / 100.0;
            // Create new SizeArgs struct
            let size = SizeArgs {
                width: 0,
                height: 0,
                scale,
            };

            process_directory(source_path, size, filter, |path, size, filter| {
                resize_by_scale(path, size.scale, filter);
            });
            return;
        }
    }

    // parse size value to u32.
    let width: u32 = size[0].parse().unwrap();
    let height: u32 = size[1].parse().unwrap();
    // Create new SizeArgs struct
    let size = SizeArgs {
        width,
        height,
        scale: 0.0,
    };

    process_directory(source_path.clone(), size, filter, |path, size, filter| {
        resize_by_size(path, (size.width, size.height), filter);
    });
}

fn main() {
    // Parse command line arguments
    let cli = CLI::parse();

    // Get source path
    let source_path = cli.source_path;
    // Get task name
    let task = cli.task;
    // get task options
    let options = cli.options;

    // check if source path is a directory.
    if !source_path.is_dir() {
        panic!("Source path is not a directory: {}", source_path.to_str().unwrap());
    }

    // parse options
    // split options by comma
    let options: Vec<&str> = options.split(",").collect();
    // split each option by equal sign
    let mut options_map = std::collections::HashMap::new();
    for option in options {
        let option: Vec<&str> = option.split("=").collect();
        if option.len() != 2 {
            panic!("Invalid option: {}", option.join("="));
        }
        // if option is empty, error out
        if option[0].is_empty() {
            panic!("Invalid option: {}", option.join("="));
        }
        // if option is empty, error out
        if option[1].is_empty() {
            panic!("Invalid option: {}", option.join("="));
        }
        options_map.insert(option[0], option[1]);
    }

    // if task is equal to resize
    if task == "resize" {
        resize(source_path, options_map);
    }
}
