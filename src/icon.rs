use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use image::{DynamicImage, ImageFormat};

/// List of output icon sizes and filenames.
const ICON_SIZES: &[(&str, u32, u32)] = &[
    ("128x128.png", 128, 128),
    ("128x128@2x.png", 256, 256),
    ("32x32.png", 32, 32),
    ("Square107x107Logo.png", 107, 107),
    ("Square142x142Logo.png", 142, 142),
    ("Square150x150Logo.png", 150, 150),
    ("Square284x284Logo.png", 284, 284),
    ("Square30x30Logo.png", 30, 30),
    ("Square310x310Logo.png", 310, 310),
    ("Square44x44Logo.png", 44, 44),
    ("Square71x71Logo.png", 71, 71),
    ("Square89x89Logo.png", 89, 89),
    ("StoreLogo.png", 50, 50),
    ("icon.png", 1024, 1024),
];

/// Converts a 1024x1024 PNG to various icon sizes.
pub fn generate_pngs(input_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input_path)?;
    let output_dir = "icons";
    std::fs::create_dir_all(output_dir)?;

    for (filename, width, height) in ICON_SIZES {
        let resized = img.resize_exact(*width, *height, image::imageops::Lanczos3);
        let output_path = Path::new(output_dir).join(filename);
        let file = File::create(&output_path)?;
        let mut writer = BufWriter::new(file);
        // write_to expects an ImageFormat enum
        resized.write_to(&mut writer, ImageFormat::Png)?;
    }

    // Generate .ico and .icns files
    generate_ico(&img, output_dir)?;

    Ok(())
}

/// Generates a Windows .ico file with multiple sizes.
fn generate_ico(img: &DynamicImage, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sizes = [16, 32, 48, 64, 128, 256];
    let mut images = Vec::new();
    for size in sizes {
        let resized = img.resize_exact(size, size, image::imageops::Lanczos3);
        images.push(resized);
    }
    let output_path = Path::new(output_dir).join("icon.ico");
    let file = File::create(output_path)?;
    let mut encoder = ico::IconDir::new(ico::ResourceType::Icon);
    for image in images {
        let icon_image = ico::IconImage::from_rgba_data(image.width(), image.height(), image.to_rgba8().into_raw());
        encoder.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }
    encoder.write(file)?;
    Ok(())
}



// Add dependencies to Cargo.toml:
// image = "0.24"
// ico = "0.3"
// icns = "0.2"