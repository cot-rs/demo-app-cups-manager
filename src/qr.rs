/// Generates a QR code from the given data into SVG XML format.
pub fn generate_qr_code<D: AsRef<[u8]>>(data: D) -> qrcode::QrResult<String> {
    let qr = qrcode::QrCode::new(data)?;
    let image = qr.render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .build();
    Ok(image)
}

/// Scans a QR code from the given image data and returns the decoded text.
pub fn scan_qr_code<D: AsRef<[u8]>>(data: D) -> Result<String, Box<dyn std::error::Error>> {
    // open the image from disk
    let img = image::load_from_memory(data.as_ref())?;

    // convert to gray scale
    let img_gray = img.into_luma8();

    // create a decoder
    let mut decoder = quircs::Quirc::default();

    // identify all qr codes
    let codes = decoder.identify(img_gray.width() as usize, img_gray.height() as usize, &img_gray);

    for code in codes {
        let decoded = code?.decode()?;
        let text = String::from_utf8(decoded.payload)?;
        return Ok(text);
    }

    Err("No QR code found".into())
}

#[cfg(test)]
mod tests {
    use image::EncodableLayout;

    #[test]
    fn generate_qr_code() {
        let data = "Hello, world!";
        let qr_code = super::generate_qr_code(data).unwrap();
        assert!(!qr_code.is_empty());
        println!("{}", qr_code);

        use resvg::usvg;
        use resvg::tiny_skia;

        let rtree = usvg::Tree::from_data(&qr_code.as_bytes(), &usvg::Options::default()).unwrap();
        let pixmap_size = rtree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        resvg::render(
            &rtree,
            tiny_skia::Transform::from_scale(1.0, 1.0),
            &mut pixmap.as_mut(),
        );

        let parsed = image::RgbaImage::from_raw(
            pixmap_size.width(),
            pixmap_size.height(),
            pixmap.data().to_vec(),
        ).expect("could not construct an image");

        let mut png = std::io::Cursor::new(Vec::new());
        parsed.write_to(&mut png, image::ImageFormat::Png).unwrap();

        let decoded = super::scan_qr_code(png.get_ref()).unwrap();
        assert_eq!(decoded, data);
    }
}
