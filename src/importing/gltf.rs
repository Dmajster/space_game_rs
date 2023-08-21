use std::{collections::BTreeMap, fs, path::Path};
use gltf::{accessor::DataType, Gltf};
use crate::VertexAttributeType;



pub fn load<P>(path: P)
where
    P: AsRef<Path>,
{
    let gltf = Gltf::open(path.as_ref().clone()).unwrap();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mut attributes = BTreeMap::new();

            for (semantic, accessor) in primitive.attributes() {
                match semantic {
                    gltf::Semantic::Positions => {
                        if accessor.data_type() != DataType::F32 {
                            todo!()
                        }

                        attributes.insert(
                            VertexAttributeType::Position,
                            get_data_from_accessor(path.as_ref().clone(), &accessor),
                        );
                    }
                    gltf::Semantic::Normals => {
                        if accessor.data_type() != DataType::F32 {
                            todo!()
                        }

                        attributes.insert(
                            VertexAttributeType::Normal,
                            get_data_from_accessor(path.as_ref().clone(), &accessor),
                        );
                    }
                    gltf::Semantic::Tangents => {
                        if accessor.data_type() != DataType::F32 {
                            todo!()
                        }

                        attributes.insert(
                            VertexAttributeType::Tangent,
                            get_data_from_accessor(path.as_ref().clone(), &accessor),
                        );
                    }
                    gltf::Semantic::Colors(i) => {
                        if accessor.data_type() != DataType::U32 {
                            todo!()
                        }

                        if i > 0 {
                            todo!()
                        }

                        attributes.insert(
                            VertexAttributeType::Color0,
                            get_data_from_accessor(path.as_ref().clone(), &accessor),
                        );
                    }
                    gltf::Semantic::TexCoords(i) => {
                        if accessor.data_type() != DataType::F32 {
                            todo!()
                        }

                        if i > 0 {
                            todo!()
                        }

                        attributes.insert(
                            VertexAttributeType::Uv0,
                            get_data_from_accessor(path.as_ref().clone(), &accessor),
                        );
                    }
                    gltf::Semantic::Joints(_) => todo!(),
                    gltf::Semantic::Weights(_) => todo!(),
                }
            }

            let indices =
                get_data_from_accessor(path.as_ref().clone(), &primitive.indices().unwrap());
        }
    }
}

fn get_data_from_accessor<P>(path: P, accessor: &gltf::Accessor) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let view = accessor.view().unwrap();
    let buffer = view.buffer();

    let buffer_data = match buffer.source() {
        gltf::buffer::Source::Bin => todo!(),
        gltf::buffer::Source::Uri(uri) => {
            fs::read(path.as_ref().parent().unwrap().join(uri)).unwrap()
        }
    };

    let from = view.offset() + accessor.offset();
    let to = view.offset() + accessor.offset() + accessor.count() * accessor.size();

    let view_data = &buffer_data[from..to];

    let data = match view.stride() {
        Some(stride) => view_data
            .iter()
            .enumerate()
            .filter_map(|(index, byte)| {
                if index % stride < accessor.size() {
                    Some(*byte)
                } else {
                    None
                }
            })
            .collect::<Vec<u8>>(),
        None => view_data.to_vec(),
    };
    data
}

// fn transmute_byte_vec<T>(mut bytes: Vec<u8>) -> Vec<T> {
//     unsafe {
//         let size_of_t = mem::size_of::<T>();
//         let length = bytes.len() / size_of_t;
//         let capacity = bytes.capacity() / size_of_t;
//         let mutptr = bytes.as_mut_ptr() as *mut T;
//         mem::forget(bytes);

//         Vec::from_raw_parts(mutptr, length, capacity)
//     }
// }
