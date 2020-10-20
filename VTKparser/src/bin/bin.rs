extern crate vtk_parser;

fn main() {
    let data = vtk_parser::read_file("tooth.vtk");

    let data = data.unwrap();

    let header = data.header;

    assert_eq!(header.file_version.0, 3);
    assert_eq!(header.file_version.1, 0);

    assert_eq!(header.header, "created by VTKUInt8Writer.py\n");

    assert_eq!(header.binary, true);

    let data = match data.dataset {
        vtk_parser::Datatype::StructuredPoints(x) => x,
        _ => panic!(),
    };

    assert_eq!(data.dims, (103, 94, 161));

    assert_eq!(data.origin, (0.0, 0.0, 0.0));

    assert_eq!(data.spacing, (1.0, 1.0, 1.0));

    //assert_eq!(data.data.len(), 1558802);
}
