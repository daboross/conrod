//! A demonstration using winit to provide events and glium for drawing the Ui.
//!
//! Note that the `glium` crate is re-exported via the `conrod::backend::glium` module.
#![cfg(all(feature="winit", feature="glium", feature="nightly"))]
#![feature(test)]

#[macro_use]
extern crate conrod;

extern crate test;
extern crate find_folder;
extern crate image;

#[path="../examples/support/mod.rs"]
mod support;

use conrod::backend::glium::glium;
use conrod::backend::glium::glium::DisplayBuild;

use test::Bencher;

// The initial width and height in "points".
const WIN_W: u32 = support::WIN_W;
const WIN_H: u32 = support::WIN_H;

fn setup()
    -> (glium::backend::glutin_backend::GlutinFacade,
        conrod::Ui,
        support::Ids,
        conrod::image::Map<glium::texture::Texture2d>,
        support::DemoApp,
        conrod::backend::glium::Renderer)
{

    // Build the window.
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(WIN_W, WIN_H)
        .with_title("Conrod with glium!")
        //.with_multisampling(8)
        .build_glium()
        .unwrap();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();

    // The `widget::Id` of each widget instantiated in `support::gui`.
    let ids = support::Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Load the Rust logo from our assets folder to use as an example image.
    fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let path = assets.join("images/rust.png");
        let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(rgba_image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
        texture
    }

    let mut image_map = conrod::image::Map::new();
    let rust_logo = image_map.insert(load_rust_logo(&display));

    // A demonstration of some app state that we want to control with the conrod GUI.
    let app = support::DemoApp::new(rust_logo);

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    //
    // Internally, the `Renderer` maintains:
    // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
    // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
    // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
    // `conrod::render::Primitive`s.
    // - a `Vec` of commands that describe how to draw the vertices.
    let renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    (display, ui, ids, image_map, app, renderer)
}

#[bench]
pub fn bench_creating_widgets(b: &mut Bencher) {
    let (_display, mut ui, ids, _image_map, mut app, _renderer) = setup();

    // Start the loop:
    //
    // - Create widgets
    // - Request redraw
    // - Transform widgets into drawing primitives.

    b.iter(|| {
        support::gui(&mut ui.set_widgets(), &ids, &mut app);
    });
}

#[bench]
pub fn bench_creating_and_drawing_widgets(b: &mut Bencher) {
    let (display, mut ui, ids, image_map, mut app, mut renderer) = setup();

    // Start the loop:
    //
    // - Create widgets
    // - Request redraw
    // - Transform widgets into drawing primitives.

    b.iter(|| {
        support::gui(&mut ui.set_widgets(), &ids, &mut app);
        ui.needs_redraw();
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
        }
    });
}
