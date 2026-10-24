//! Gera `crates/app/assets/brand/icon-<n>.png` e `icon.ico` a partir de `icon.svg`.

use resvg::{tiny_skia, usvg};

const SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

fn render(tree: &usvg::Tree, size: u32) -> image::RgbaImage {
    let mut pix = tiny_skia::Pixmap::new(size, size).expect("tamanho válido");
    let s = size as f32 / tree.size().width();
    resvg::render(tree, tiny_skia::Transform::from_scale(s, s), &mut pix.as_mut());
    // tiny-skia usa alfa pré-multiplicado; PNG/ICO esperam alfa direto
    let mut img = image::RgbaImage::new(size, size);
    for (dst, p) in img.pixels_mut().zip(pix.pixels()) {
        let c = p.demultiply();
        *dst = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
    }
    img
}

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/app/assets/brand");
    let svg = std::fs::read(dir.join("icon.svg")).expect("icon.svg");
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default()).expect("SVG válido");
    let mut frames = Vec::new();
    for n in SIZES {
        let img = render(&tree, n);
        img.save(dir.join(format!("icon-{n}.png"))).expect("salvar png");
        frames.push(image::codecs::ico::IcoFrame::as_png(img.as_raw(), n, n, image::ExtendedColorType::Rgba8).expect("frame"));
    }
    render(&tree, 1024).save(dir.join("icon-1024.png")).expect("salvar png");
    let f = std::fs::File::create(dir.join("icon.ico")).expect("ico");
    image::codecs::ico::IcoEncoder::new(f).encode_images(&frames).expect("encode ico");
    println!("ok: {}", dir.display());
}
