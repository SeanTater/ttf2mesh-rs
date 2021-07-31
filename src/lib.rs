#![feature(test)]
extern crate test;

use std::{ffi::CString, os::unix::prelude::OsStrExt, path::Path};

mod error;
mod glyph;
mod mesh;
mod outputs;
mod quality;
mod ttf;

pub use error::Error;
pub use glyph::Glyph;
pub use mesh::{Mesh2d, Mesh3d};
pub use quality::Quality;
pub use ttf::TTFFile;

// TODO: support TTF_FEATURE_IGN_ERR as bitflag

fn path_to_cstring<P: AsRef<Path>>(path: P) -> CString {
    CString::new(path.as_ref().as_os_str().as_bytes()).unwrap()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use test::Bencher;

    fn get_font_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts")
    }

    fn get_font(font_file: Option<&str>) -> PathBuf {
        match font_file {
            Some(file) => get_font_path().join(file),
            None => get_font_path().join("FiraMono-Medium.ttf"),
        }
    }

    fn read_font(font_file: Option<&str>) -> Vec<u8> {
        std::fs::read(get_font(font_file)).unwrap()
    }

    #[test]
    fn test_from_buffer_vec() {
        let _ = TTFFile::from_buffer_vec(read_font(None)).unwrap();
    }

    #[test]
    fn test_from_file() {
        let _ = TTFFile::from_file(get_font(None)).unwrap();
    }

    #[test]
    fn test_get_glyph_by_char() {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();
        let char = "A".chars().next().unwrap();
        let _ = font.glyph_by_char(char).unwrap();
    }

    #[test]
    fn test_to_3d_mesh() {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();
        let mut glyph = font.glyph_by_char("€".chars().next().unwrap()).unwrap();
        let mesh = glyph.to_3d_mesh(Quality::Low, 0.5).unwrap();

        let mut sizes = Vec::new();
        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_normals().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        let mesh = glyph.to_3d_mesh(Quality::High, 1.5).unwrap();

        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_normals().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        let mesh = glyph.to_3d_mesh(Quality::Custom(255), 0.5).unwrap();

        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_normals().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        assert_eq!(sizes, &[246, 246, 160, 552, 552, 364, 1164, 1164, 772]);
    }

    #[test]
    fn test_to_2d_mesh() {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();
        let mut glyph = font.glyph_by_char("€".chars().next().unwrap()).unwrap();

        let mut sizes = Vec::new();
        let mesh = glyph.to_2d_mesh(Quality::Low).unwrap();
        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        let mesh = glyph.to_2d_mesh(Quality::High).unwrap();
        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        let mesh = glyph.to_2d_mesh(Quality::Custom(255)).unwrap();
        sizes.extend_from_slice(&[
            mesh.iter_vertices().collect::<Vec<_>>().len(),
            mesh.iter_faces().collect::<Vec<_>>().len(),
        ]);

        assert_eq!(sizes, &[41, 39, 92, 90, 194, 192]);
    }

    #[bench]
    fn bench_open_font(b: &mut Bencher) {
        let buffer = read_font(None);

        b.iter(|| {
            let _ = TTFFile::from_buffer_vec(buffer.clone()).unwrap();
        });
    }

    #[bench]
    fn bench_get_glyph(b: &mut Bencher) {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();
        let char = "€".chars().next().unwrap();

        b.iter(|| {
            let _ = font.glyph_by_char(char).unwrap();
        });
    }

    #[bench]
    fn bench_glyph_to_3d_mesh_low_quality(b: &mut Bencher) {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();

        let char = "€".chars().next().unwrap();
        let mut glyph = font.glyph_by_char(char).unwrap();

        b.iter(|| {
            let _ = glyph.to_3d_mesh(Quality::Low, 0.1).unwrap();
        });
    }

    #[bench]
    fn bench_glyph_to_3d_mesh_high_quality(b: &mut Bencher) {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();

        let char = "€".chars().next().unwrap();
        let mut glyph = font.glyph_by_char(char).unwrap();

        b.iter(|| {
            let _ = glyph.to_3d_mesh(Quality::High, 0.1).unwrap();
        });
    }

    #[bench]
    fn bench_glyph_to_2d_mesh_low_quality(b: &mut Bencher) {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();

        let char = "€".chars().next().unwrap();
        let mut glyph = font.glyph_by_char(char).unwrap();

        b.iter(|| {
            let _ = glyph.to_2d_mesh(Quality::Low).unwrap();
        });
    }

    #[bench]
    fn bench_glyph_to_2d_mesh_high_quality(b: &mut Bencher) {
        let mut font = TTFFile::from_buffer_vec(read_font(None)).unwrap();

        let char = "€".chars().next().unwrap();
        let mut glyph = font.glyph_by_char(char).unwrap();

        b.iter(|| {
            let _ = glyph.to_2d_mesh(Quality::High).unwrap();
        });
    }
}
