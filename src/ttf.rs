use core::slice;

use std::{convert::TryInto, mem::MaybeUninit, path::Path};

use ttf2mesh_sys as sys;

use crate::{path_to_cstring, Error, Glyph, Quality};

/// A decoded TTF file instance
///
/// Usage - opening a file:
/// ```rust
/// # use ttf2mesh::{TTFFile, Quality};
///
/// // from a file
/// let ttf = TTFFile::from_file("./fonts/FiraMono-Medium.ttf").unwrap();
///
/// // from a buffer
/// let my_vec = std::fs::read("./fonts/FiraMono-Medium.ttf").unwrap();
/// let mut ttf = TTFFile::from_buffer_vec(my_vec).unwrap();
///
/// // how many fonts?
/// assert_eq!(ttf.glyph_count(), 1485);
///
/// // export all glyphs as 2d meshes to a .obj file
/// ttf.export_to_obj("/dev/null", Quality::Low).unwrap();
///
/// // generate 2d mesh for a glyph
/// let mut glyph = ttf.glyph_from_char('€').unwrap();
/// let mesh = glyph.to_2d_mesh(Quality::Medium).unwrap();
/// assert_eq!(mesh.vertices_len(), 56);
/// assert_eq!(mesh.iter_vertices().next().unwrap().value(), (0.555, 0.656));
///
/// assert_eq!(mesh.faces_len(), 54);
/// assert_eq!(mesh.iter_faces().next().unwrap().value(), (53, 52, 5));
/// ```
pub struct TTFFile {
    ttf: *mut sys::ttf_file,
}

impl TTFFile {
    /// Load TTF font from a memory buffer
    ///
    /// Has to take ownership since the buffer is being modified at runtime
    pub fn from_buffer_vec(data: Vec<u8>) -> Result<TTFFile, Error> {
        let mut ttf = MaybeUninit::uninit();
        let error = unsafe {
            sys::ttf_load_from_mem(
                data.as_ptr(),
                data.len().try_into().unwrap(),
                ttf.as_mut_ptr(),
                false,
            )
        };
        Self::load(ttf, error)
    }

    /// Load TTF font from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<TTFFile, Error> {
        if !Path::new(path.as_ref().as_os_str()).exists() {
            return Err(Error::FileNotFound);
        }

        let file_name = path_to_cstring(path);

        let mut ttf = MaybeUninit::uninit();
        let error = unsafe { sys::ttf_load_from_file(file_name.as_ptr(), ttf.as_mut_ptr(), false) };

        Self::load(ttf, error)
    }

    fn load(ttf: MaybeUninit<*mut sys::ttf_file>, error: i32) -> Result<TTFFile, Error> {
        if error != ttf2mesh_sys::TTF_DONE as i32 {
            // fprintf(stderr, "Unable to load font: %s\n", ttf_error_str[error]);
            return Err(Error::FontLoadError);
        }

        Ok(Self {
            ttf: unsafe { ttf.assume_init() },
        })
    }

    /// Export all glyphs to a .obj -file
    pub fn export_to_obj<P: AsRef<Path>>(
        &mut self,
        obj_path: P,
        quality: Quality,
    ) -> Result<(), Error> {
        let file_name = path_to_cstring(obj_path);

        let error =
            unsafe { sys::ttf_export_to_obj(self.ttf, file_name.as_ptr(), quality.as_u8()) };

        if error != ttf2mesh_sys::TTF_DONE as i32 {
            // fprintf(stderr, "Unable to export font: %s\n", ttf_error_str[error]);
            return Err(Error::ObjExportError);
        }

        Ok(())
    }

    /// Get a glyph for a character
    pub fn glyph_from_char<'a>(&'a mut self, char: char) -> Result<Glyph<'a>, Error> {
        let mut bytes = [0; 2];
        char.encode_utf16(&mut bytes);

        let index = unsafe { sys::ttf_find_glyph(self.ttf, bytes[0]) };

        if index < 0 {
            return Err(Error::GlyphNotFound);
        }

        self.glyph_by_index(index.try_into().unwrap())
    }

    /// Total count of glyphs in a ttf file
    pub fn glyph_count(&self) -> usize {
        unsafe { *self.ttf }.nglyphs.try_into().unwrap()
    }

    /// Get a glyph by its index. See also [`glyph_from_char`]
    pub fn glyph_by_index<'a>(&'a mut self, index: usize) -> Result<Glyph<'a>, Error> {
        let glyphs = unsafe { slice::from_raw_parts_mut((*self.ttf).glyphs, self.glyph_count()) };

        match glyphs.get_mut(index) {
            Some(glyph) => Ok(Glyph::from_raw(glyph)),
            None => Err(Error::GlyphNotFound),
        }
    }
}

impl Drop for TTFFile {
    fn drop(&mut self) {
        unsafe { sys::ttf_free(self.ttf) }
    }
}
