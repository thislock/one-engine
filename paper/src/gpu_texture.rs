use std::{collections::HashMap, sync::Arc};

use anyhow::{Error, Ok};
use image::GenericImageView;
use wgpu::{BindGroup, BindGroupLayout};

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl Texture {

    pub fn from_bytes(
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> anyhow::Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(texture_bind_group_layout, device, queue, &img, Some(label))
    }

    pub fn from_image(
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        Ok(Self {
            texture,
            diffuse_bind_group,
        })
    }
}

pub struct TextureBundle {
    fallback_texture: Arc<Texture>,
    
    textures: HashMap<String, Arc<Texture>>,
    texture_bind_group_layout: BindGroupLayout,
}

impl TextureBundle {

    pub fn get_texture_bind_group(&self) -> &BindGroupLayout {
        return &self.texture_bind_group_layout;
    }

    pub fn get_diffuse_bind_group(&self, label: &str) -> &BindGroup {
        let label = String::from(label);
        if self.textures.contains_key(&label) {
            return &self.textures.get(&label).unwrap().diffuse_bind_group;
        } else {
            return &self.fallback_texture.diffuse_bind_group;
        }
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Self> {
        
        let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                    },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        
        // hard coded for stability
        use crate::z_missing_texture::*;
        let tex_data: image::RgbaImage = image::ImageBuffer::from_raw(
            FALLBACK_TEXTURE_WIDTH, 
            FALLBACK_TEXTURE_HEIGHT, 
            FALLBACK_TEXTURE_DATA.to_vec(),
        ).expect("FATAL ERROR, PUT BACK THE HARDCODED FALLBACK TEXTURE, BOZO");

        let fallback_texture = Texture::from_image(
            &texture_bind_group_layout, device, queue, &image::DynamicImage::from(tex_data), Some("Fallback Texture"))?;

        Ok(Self {
            textures: HashMap::new(),
            fallback_texture: Arc::from(fallback_texture),
            texture_bind_group_layout,
        })
    }

    pub fn add_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> anyhow::Result<()> {
        let label = String::from(label);

        if self.textures.contains_key(&label) {
            let mut error_message: String = String::from("this texture already exists: ");
            error_message.push_str(&label);
            return Err(Error::msg(error_message));
        }
        
        let tex = Texture::from_bytes(&self.texture_bind_group_layout, device, queue, bytes, &label)?;
        self.textures.insert(label, Arc::new(tex));

        return Ok(());
    }
}