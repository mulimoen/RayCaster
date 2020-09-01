use std::io;
use std::io::prelude::*;

use crate::VTKparseError;

pub struct StructuredPoints {
    pub dims: (u32, u32, u32),
    pub origin: (f32, f32, f32),
    pub spacing: (f32, f32, f32),
    pub data: Vec<u8>,
    pub datatype: String,
    pub dataname: String,
    pub numcomp: u8,
    pub tablename: String,
}

pub fn parse<R: io::Read>(
    reader: &mut io::BufReader<R>,
    binary: bool,
) -> Result<StructuredPoints, VTKparseError> {
    let mut dims = String::new();
    reader.read_line(&mut dims)?;

    if !dims.starts_with("DIMENSIONS ") {
        return Err(VTKparseError::FileFormat(
            "Does not contain dimensions".to_string(),
        ));
    }
    let dims = get_dimensions(&dims)?;

    let mut origin = String::new();
    reader.read_line(&mut origin)?;
    if !origin.starts_with("ORIGIN ") {
        return Err(VTKparseError::FileFormat(
            "Does not contain origin".to_string(),
        ));
    }
    let origo = get_origo(&origin)?;

    let mut spacing = String::new();
    reader.read_line(&mut spacing)?;

    if !spacing.starts_with("SPACING ") && !spacing.starts_with("ASPECT_RATIO ") {
        return Err(VTKparseError::FileFormat(
            "Does not contain spacing".to_string(),
        ));
    }
    let spacing = get_spacing(&spacing)?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    if !line.starts_with("POINT_DATA ") {
        return Err(VTKparseError::FileFormat(
            "Does not contain length".to_string(),
        ));
    }

    let len: usize = line.split_whitespace().last().unwrap().parse()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    if !line.starts_with("SCALARS ") {
        return Err(VTKparseError::FileFormat(
            "Does not contain scalar type".to_string(),
        ));
    }

    let mut words = line.split_whitespace().skip(1);

    let dataname = words
        .next()
        .ok_or_else(|| VTKparseError::FileFormat("Could not find dataname".to_string()))?;
    let datatype = words
        .next()
        .ok_or_else(|| VTKparseError::FileFormat("Could not find datatype".to_string()))?;
    let numcomp: u8 = words.next().unwrap_or("1").parse()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;

    if !line.starts_with("LOOKUP_TABLE") {
        return Err(VTKparseError::FileFormat("No lookup table".to_string()));
    }

    let mut table = line.split_whitespace().skip(1);
    let tablename = table.next().unwrap_or("default");

    if datatype != "unsigned_char" {
        return Err(VTKparseError::NotImplemented(
            format!("Datatype {}", datatype).to_string(),
        ));
    }

    if !binary {
        return Err(VTKparseError::NotImplemented(
            "ASCII extraction".to_string(),
        ));
    }

    let mut colours = Vec::with_capacity(len);
    reader.read_to_end(&mut colours)?;

    if colours.len() != len {
        return Err(VTKparseError::UnknownFormat(format!(
            "Number of elements in binary is not correct. Expected {} but got {}",
            len,
            colours.len()
        )));
    }

    Ok(StructuredPoints {
        dims: dims,
        origin: origo,
        spacing: spacing,
        data: colours,
        datatype: datatype.to_string(),
        dataname: dataname.to_string(),
        numcomp: numcomp,
        tablename: tablename.to_string(),
    })
}

fn get_dimensions(dims: &str) -> Result<(u32, u32, u32), VTKparseError> {
    let mut dims = dims.split_whitespace().skip(1);

    let d0 = dims.next();
    let d1 = dims.next();
    let d2 = dims.next();

    match (d0, d1, d2) {
        (Some(x), Some(y), Some(z)) => Ok((x.parse()?, y.parse()?, z.parse()?)),
        _ => Err(VTKparseError::FileFormat(
            "Can not extract dimensions".to_string(),
        )),
    }
}

fn get_origo(origin: &str) -> Result<(f32, f32, f32), VTKparseError> {
    let mut origin = origin.split_whitespace().skip(1);

    let d0 = origin.next();
    let d1 = origin.next();
    let d2 = origin.next();

    match (d0, d1, d2) {
        (Some(x), Some(y), Some(z)) => Ok((x.parse()?, y.parse()?, z.parse()?)),
        _ => Err(VTKparseError::FileFormat(
            "Can not extract origin".to_string(),
        )),
    }
}

fn get_spacing(spacing: &str) -> Result<(f32, f32, f32), VTKparseError> {
    let mut spacing = spacing.split_whitespace().skip(1);

    let d0 = spacing.next();
    let d1 = spacing.next();
    let d2 = spacing.next();

    match (d0, d1, d2) {
        (Some(x), Some(y), Some(z)) => Ok((x.parse()?, y.parse()?, z.parse()?)),
        _ => Err(VTKparseError::FileFormat(
            "Can not extract origin".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_something() {
        use read_file;
        read_file("../data/tooth.vtk").unwrap();
    }
}
