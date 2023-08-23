use glam::{Vec2, Vec3};
use gltf::{Gltf, Semantic};
use std::{fs, mem, path::Path};

use crate::Mesh;

pub fn load<P>(path: &P)
where
    P: AsRef<Path>,
{
    let gltf = Gltf::open(&path).unwrap();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let positions = primitive
                .attributes()
                .find_map(|(semantic, accessor)| {
                    if semantic == Semantic::Positions {
                        Some(get_data_from_accessor::<P, Vec3>(&path, &accessor))
                    } else {
                        None
                    }
                })
                .unwrap();

            let normals = primitive
                .attributes()
                .find_map(|(semantic, accessor)| {
                    if semantic == Semantic::Normals {
                        Some(get_data_from_accessor::<P, Vec3>(&path, &accessor))
                    } else {
                        None
                    }
                })
                .unwrap();

            let uvs = primitive
                .attributes()
                .find_map(|(semantic, accessor)| {
                    if semantic == Semantic::TexCoords(0) {
                        Some(get_data_from_accessor::<P, Vec2>(&path, &accessor))
                    } else {
                        None
                    }
                })
                .unwrap();

            let indices = get_data_from_accessor::<P, u32>(&path, &primitive.indices().unwrap());

            let mesh = Mesh {
                positions,
                normals,
                uvs,
                indices,
            };
        }
    }
}

fn get_data_from_accessor<P, T>(path: &P, accessor: &gltf::Accessor) -> Vec<T>
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

    transmute_byte_vec::<T>(data)
}

fn transmute_byte_vec<T>(mut bytes: Vec<u8>) -> Vec<T> {
    unsafe {
        let size_of_t = mem::size_of::<T>();
        let length = bytes.len() / size_of_t;
        let capacity = bytes.capacity() / size_of_t;
        let mutptr = bytes.as_mut_ptr() as *mut T;
        mem::forget(bytes);

        Vec::from_raw_parts(mutptr, length, capacity)
    }
}
