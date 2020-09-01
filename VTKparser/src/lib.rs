use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub use error::VTKparseError;
pub use spoints::StructuredPoints;

mod error;
mod spoints;

pub struct Data {
    pub header: Header,
    pub dataset: Datatype,
}

pub struct Header {
    pub file_version: (usize, usize),
    pub header: String,
    pub binary: bool,
}

pub enum Datatype {
    StructuredPoints(StructuredPoints),
    StructuredGrid,
    RectilinearGrid,
    PolyData,
    UnstructuredGrid,
    Field,
    Empty,
}

pub enum DatasetAttributes {
    Scalars,
    LookupTable,
    Vectors,
    Normals,
    TextureCoordinates,
    Tensors,
    FieldData,
}

impl Data {
    pub fn structured_points(&self) -> Option<&StructuredPoints> {
        if let Datatype::StructuredPoints(ref x) = self.dataset {
            return Some(x);
        }
        None
    }
}

pub fn read_file<P: AsRef<Path>>(file: P) -> Result<Data, VTKparseError> {
    let file = File::open(file)?;
    let mut reader = BufReader::new(file);

    let mut version = String::new();
    reader.read_line(&mut version)?;

    let version = get_version(&version)?;

    let mut header = String::with_capacity(256);
    reader.read_line(&mut header)?;

    let mut file_format = String::new();
    reader.read_line(&mut file_format)?;

    let binary = if file_format == "ASCII\n" {
        Ok(false)
    } else if file_format == "BINARY\n" {
        Ok(true)
    } else {
        Err(VTKparseError::FileFormat(
            "Format is not ASCII or BINARY".to_string(),
        ))
    }?;

    Ok(Data {
        header: Header {
            file_version: version,
            header: header,
            binary: binary,
        },
        dataset: dataset_parse(reader, binary)?,
    })
}

fn get_version(version: &str) -> Result<(usize, usize), VTKparseError> {
    if !version.starts_with("# vtk DataFile Version ") {
        return Err(VTKparseError::UnknownFormat(
            "Identifier is not recognized".to_string(),
        ));
    }

    let mut v = version
        .split_whitespace()
        .last()
        .unwrap_or("0.0")
        .split('.');
    let d0 = v.next().unwrap_or("0");
    let d1 = v.next().unwrap_or("0");
    Ok((d0.parse()?, d1.parse()?))
}

fn dataset_parse<R: Read>(
    mut reader: BufReader<R>,
    binary: bool,
) -> Result<Datatype, VTKparseError> {
    let mut dataset = String::new();
    reader.read_line(&mut dataset)?;

    if !dataset.starts_with("DATASET ") {
        return Ok(Datatype::Empty);
    }

    let vtk_type = dataset.split_whitespace().last().unwrap().to_uppercase();

    if vtk_type == "STRUCTURED_POINTS" {
        Ok(Datatype::StructuredPoints(spoints::parse(
            &mut reader,
            binary,
        )?))
    } else if vtk_type == "STRUCTURED_GRID" {
        Ok(Datatype::StructuredGrid)
    } else if vtk_type == "RECTILINEAR_GRID" {
        Ok(Datatype::RectilinearGrid)
    } else if vtk_type == "POLYGONAL_DATA" {
        Ok(Datatype::PolyData)
    } else if vtk_type == "UNSTRUCTURED_GRID" {
        Ok(Datatype::UnstructuredGrid)
    } else if vtk_type == "FIELD" {
        Ok(Datatype::Field)
    } else {
        Err(VTKparseError::FileFormat(
            "Unknown dataset type".to_string(),
        ))
    }
}
