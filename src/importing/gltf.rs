use glam::{Vec2, Vec3};
use gltf::{accessor::DataType, buffer, image, texture, Primitive, Semantic};
use std::{mem, path::Path};

use crate::{
    asset_server::{Asset, AssetMetadata, AssetServer, asset_id::AssetId},
    rendering::{Material, Mesh, Model, Texture},
};

//TODO: Remove duplicates
//TODO: Zeux's mesh optimizer https://github.com/zeux/meshoptimizer (but as a processor step not loader)
pub fn load<P>(path: &P, asset_server: &AssetServer)
where
    P: AsRef<Path>,
{
    let (gltf, buffers, images) = gltf::import(&path).unwrap();

    let mut models = asset_server.models_mut();
    let mut meshes = asset_server.meshes_mut();

    for mesh in gltf.meshes() {
        let model_name = if let Some(mesh_name) = mesh.name() {
            Some(format!("{}", mesh_name.to_owned()))
        } else {
            None
        };

        let model_asset = models.add(Model::default(), AssetMetadata { name: model_name });

        for primitive in mesh.primitives() {
            let mesh_name = if let Some(mesh_name) = mesh.name() {
                Some(format!("{}_{}", mesh_name.to_owned(), primitive.index()))
            } else {
                None
            };

            let mesh = create_primitive(&buffers, &primitive);
            let mesh_metadata = AssetMetadata { name: mesh_name };

            get_or_create_material(&primitive.material(), &images, asset_server);

            let mesh_asset = meshes.add(mesh, mesh_metadata);
            model_asset.asset.mesh_ids.push(mesh_asset.id());
        }
    }
}

fn create_primitive(buffers: &Vec<buffer::Data>, primitive: &gltf::mesh::Primitive<'_>) -> Mesh {
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

fn get_or_create_asset_texture(
    info: texture::Info<'_>,
    images: &Vec<image::Data>,
    asset_server: &AssetServer,
) -> AssetId<Texture> {
    let texture = info.texture();
    let index = texture.source().index();
    let image = images.get(index).unwrap();

    let name = if let Some(texture_name) = texture.name() {
        Some(format!("{}", texture_name.to_owned()))
    } else {
        None
    };

    let mut textures = asset_server.textures_mut();
    let asset = textures.add(Texture {}, AssetMetadata { name });

    asset.id()
}

fn get_or_create_material(
    material: &gltf::material::Material,
    images: &Vec<image::Data>,
    asset_server: &AssetServer,
) -> AssetId<Material> {
    let name = if let Some(material_name) = material.name() {
        Some(format!("{}", material_name.to_owned()))
    } else {
        None
    };

    let color_texture_id =
        if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
            Some(get_or_create_asset_texture(info, images, asset_server))
        } else {
            None
        };

    let metallic_roughness_texture_id = if let Some(info) = material
        .pbr_metallic_roughness()
        .metallic_roughness_texture()
    {
        Some(get_or_create_asset_texture(info, images, asset_server))
    } else {
        None
    };

    let mut materials = asset_server.materials_mut();
    let asset = materials.add(
        Material {
            color_texture_id,
            metallic_roughness_texture_id,
        },
        AssetMetadata { name },
    );

    asset.id()
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
