use glam::{Vec2, Vec3};
use gltf::{accessor::DataType, buffer, Primitive, Semantic};
use std::{mem, path::Path};

use crate::{
    asset_server::{AssetMetadata, AssetServer},
    rendering::{Mesh, Model},
};

//TODO: Remove duplicates
//TODO: Zeux's mesh optimizer https://github.com/zeux/meshoptimizer (but as a processor step not loader)
pub fn load<P>(path: &P, asset_server: &mut AssetServer)
where
    P: AsRef<Path>,
{
    let (gltf, buffers, images) = gltf::import(&path).unwrap();

    for mesh in gltf.meshes() {
        let model_name = if let Some(mesh_name) = mesh.name() {
            Some(format!("{}", mesh_name.to_owned()))
        } else {
            None
        };

        let model_asset = asset_server
            .models
            .add(Model::default(), AssetMetadata { name: model_name });

        for primitive in mesh.primitives() {
            let mesh_name = if let Some(mesh_name) = mesh.name() {
                Some(format!("{}_{}", mesh_name.to_owned(), primitive.index()))
            } else {
                None
            };

            let mesh = load_primitive(&buffers, &primitive);
            let mesh_metadata = AssetMetadata { name: mesh_name };

            let mesh_asset = asset_server.meshes.add(mesh, mesh_metadata);
            model_asset.asset.meshes.push(mesh_asset.id());
        }
    }
}

fn load_primitive(buffers: &Vec<buffer::Data>, primitive: &gltf::mesh::Primitive<'_>) -> Mesh {
    let positions = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::Positions {
                Some(get_data_from_accessor::<Vec3>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap();

    let normals = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::Normals {
                Some(get_data_from_accessor::<Vec3>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap();

    let uvs = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::TexCoords(0) {
                Some(get_data_from_accessor::<Vec2>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap();

    let indices = match primitive.indices().unwrap().data_type() {
        DataType::U16 => get_data_from_accessor::<u16>(&buffers, &primitive.indices().unwrap())
            .into_iter()
            .map(|index| index as u32)
            .collect::<Vec<_>>(),
        DataType::U32 => get_data_from_accessor::<u32>(&buffers, &primitive.indices().unwrap()),
        _ => todo!(),
    };

    Mesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

fn get_data_from_accessor<T>(buffers: &Vec<buffer::Data>, accessor: &gltf::Accessor) -> Vec<T> {
    let view = accessor.view().unwrap();
    let buffer = view.buffer();

    let buffer_data = buffers.get(buffer.index()).unwrap();

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
