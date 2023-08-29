use ::image::{DynamicImage, RgbImage};
use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};
use gltf::{accessor::DataType, buffer, image, texture, Semantic};
use std::{mem, path::Path};

use crate::{
    asset_server::{asset_id::AssetId, AssetMetadata, AssetServer},
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

            let material_asset_id =
                get_or_create_material(&primitive.material(), &images, asset_server);

            let mesh_asset = meshes.add(mesh, mesh_metadata);
            model_asset.asset.mesh_ids.push(mesh_asset.id());
            model_asset.asset.material_ids.push(material_asset_id);
        }
    }
}

fn create_primitive(buffers: &Vec<buffer::Data>, primitive: &gltf::mesh::Primitive<'_>) -> Mesh {
    let mut mesh = Mesh::default();

    mesh.positions = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::Positions {
                Some(get_data_from_accessor::<Vec3>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap();

    mesh.indices = match primitive.indices().unwrap().data_type() {
        DataType::U16 => get_data_from_accessor::<u16>(&buffers, &primitive.indices().unwrap())
            .into_iter()
            .map(|index| index as u32)
            .collect::<Vec<_>>(),
        DataType::U32 => get_data_from_accessor::<u32>(&buffers, &primitive.indices().unwrap()),
        _ => todo!(),
    };

    mesh.uvs = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::TexCoords(0) {
                Some(get_data_from_accessor::<Vec2>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap_or(vec![Vec2::ZERO; mesh.positions.len()]);

    //TODO: encode normals, tangents and bitangents into qtangents (https://www.yosoygames.com.ar/wp/2018/03/vertex-formats-part-1-compression/)
    mesh.normals = primitive
        .attributes()
        .find_map(|(semantic, accessor)| {
            if semantic == Semantic::Normals {
                Some(get_data_from_accessor::<Vec3>(&buffers, &accessor))
            } else {
                None
            }
        })
        .unwrap_or_else(|| create_normals(&mesh));

    let encoded_tangents = primitive.attributes().find_map(|(semantic, accessor)| {
        if semantic == Semantic::Tangents {
            Some(get_data_from_accessor::<Vec4>(&buffers, &accessor))
        } else {
            None
        }
    });

    if let Some(encoded_tangents) = encoded_tangents {
        mesh.bitangents = mesh
            .normals
            .iter()
            .zip(encoded_tangents.iter())
            .map(|(normal, encoded_tangent)| {
                normal.cross(encoded_tangent.xyz() * encoded_tangent.w.signum())
            })
            .collect::<Vec<_>>();

        mesh.tangents = encoded_tangents
            .into_iter()
            .map(|encoded_tangent| encoded_tangent.xyz())
            .collect::<Vec<_>>();
    } else {
        mesh.tangents = vec![Vec3::ZERO; mesh.positions.len()];
        mesh.bitangents = vec![Vec3::ZERO; mesh.positions.len()];

        mikktspace::generate_tangents(&mut mesh);
    }

    mesh
}

fn create_normals(mesh: &Mesh) -> Vec<Vec3> {
    let mut normal_hits = vec![0; mesh.positions.len()];

    if mesh.indices.len() % 3 != 0 {
        panic!("this was made to work with triangles");
    }

    let mut normals = vec![Vec3::ZERO; mesh.positions.len()];

    for chunk in mesh.indices.chunks(3) {
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        let v0 = mesh.positions[i0];
        let v1 = mesh.positions[i1];
        let v2 = mesh.positions[i2];

        // Calculate face normal
        let n = (v1 - v0).cross(v2 - v0).normalize();

        // Add it to each vertex normal
        normals[i0] += n;
        normals[i1] += n;
        normals[i2] += n;

        // Increment how many normals are in the vertex normal
        normal_hits[i0] += 1;
        normal_hits[i1] += 1;
        normal_hits[i2] += 1;
    }

    // Divide by hit so we get an average of all the normals in the vertex normal
    normals = normals
        .into_iter()
        .zip(normal_hits.into_iter())
        .map(|(normal, hits)| normal / hits as f32)
        .collect::<Vec<_>>();

    normals
}

fn get_or_create_asset_texture(
    texture: texture::Texture<'_>,
    images: &Vec<image::Data>,
    asset_server: &AssetServer,
) -> AssetId<Texture> {
    let index = texture.source().index();
    let image = images.get(index).unwrap();

    let name = if let Some(texture_name) = texture.name() {
        Some(format!("{}", texture_name.to_owned()))
    } else {
        None
    };

    let mut textures = asset_server.textures_mut();

    let (wgpu_format, wgpu_bytes) = convert_to_valid_wgpu_format(&image);

    let asset = textures.add(
        Texture {
            width: image.width,
            height: image.height,
            format: wgpu_format,
            bytes: wgpu_bytes.clone(),
        },
        AssetMetadata { name },
    );

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
            Some(get_or_create_asset_texture(
                info.texture(),
                images,
                asset_server,
            ))
        } else {
            None
        };

    let normal_texture_id = if let Some(normal_texture) = material.normal_texture() {
        Some(get_or_create_asset_texture(
            normal_texture.texture(),
            images,
            asset_server,
        ))
    } else {
        None
    };

    let metallic_roughness_texture_id = if let Some(info) = material
        .pbr_metallic_roughness()
        .metallic_roughness_texture()
    {
        Some(get_or_create_asset_texture(
            info.texture(),
            images,
            asset_server,
        ))
    } else {
        None
    };

    let mut materials = asset_server.materials_mut();
    let asset = materials.add(
        Material {
            color_texture_id,
            normal_texture_id,
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

fn convert_to_valid_wgpu_format(image: &image::Data) -> (wgpu::TextureFormat, Vec<u8>) {
    match image.format {
        image::Format::R8 => (wgpu::TextureFormat::R8Unorm, image.pixels.clone()),
        image::Format::R8G8 => (wgpu::TextureFormat::Rg8Unorm, image.pixels.clone()),
        image::Format::R8G8B8 => {
            let rgb = RgbImage::from_vec(image.width, image.height, image.pixels.clone()).unwrap();

            (
                wgpu::TextureFormat::Rgba8Unorm,
                DynamicImage::ImageRgb8(rgb).into_rgba8().into_raw(),
            )
        }
        image::Format::R8G8B8A8 => (wgpu::TextureFormat::Rgba8Unorm, image.pixels.clone()),
        image::Format::R16 => (wgpu::TextureFormat::R16Unorm, image.pixels.clone()),
        image::Format::R16G16 => (wgpu::TextureFormat::Rg16Unorm, image.pixels.clone()),
        image::Format::R16G16B16 => unimplemented!(),
        image::Format::R16G16B16A16 => (wgpu::TextureFormat::Rgba16Unorm, image.pixels.clone()),
        image::Format::R32G32B32FLOAT => unimplemented!(),
        image::Format::R32G32B32A32FLOAT => {
            (wgpu::TextureFormat::Rgba32Float, image.pixels.clone())
        }
    }
}

impl mikktspace::Geometry for Mesh {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.indices[face * 3 + vert] as usize;

        self.positions[index].to_array()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.indices[face * 3 + vert] as usize;

        self.normals[index].to_array()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let index = self.indices[face * 3 + vert] as usize;

        self.uvs[index].to_array()
    }

    fn set_tangent(
        &mut self,
        tangent: [f32; 3],
        bi_tangent: [f32; 3],
        _f_mag_s: f32,
        _f_mag_t: f32,
        _bi_tangent_preserves_orientation: bool,
        face: usize,
        vert: usize,
    ) {
        let index = self.indices[face * 3 + vert] as usize;

        self.tangents[index] = tangent.into();
        self.bitangents[index] = bi_tangent.into();
    }
}
