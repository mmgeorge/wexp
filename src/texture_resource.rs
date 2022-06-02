use wgpu::*;
use core::num;
use image::GenericImageView;


pub struct TextureResource {
  pub texture: Texture,
  pub view : TextureView,
  pub sampler: Sampler,
}

impl TextureResource {
  pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: &str) -> Self {
    let image = image::load_from_memory(bytes).expect("Unable to load texture");

    Self::from_image(&device, queue, &image, label)
  }

  pub fn from_image(device: &Device, queue: &Queue, image: &image::DynamicImage, label: &str) -> Self {
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    let size = Extent3d { width, height, depth_or_array_layers: 1 };

    let texture = device.create_texture(&TextureDescriptor {
      label: Some(label),
      // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
      // COPY_DST means that we want to copy data to this texture
      usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8UnormSrgb, 
      mip_level_count: 1,
      sample_count: 1,
    });

    let image_copy_texture = ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All
    }; 

    let layout = ImageDataLayout {
      offset: 0,
      bytes_per_row: num::NonZeroU32::new(4 * width),
      rows_per_image: num::NonZeroU32::new(height), 
    }; 

    queue.write_texture(image_copy_texture, &rgba, layout, size);

    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
      label: Some(label), 
      // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
      // COPY_DST means that we want to copy data to this texture
      address_mode_u: AddressMode::ClampToEdge,
      address_mode_v: AddressMode::ClampToEdge,
      address_mode_w: AddressMode::ClampToEdge,
      // The mag_filter and min_filter options describe what to do when a fragment covers multiple pixels,
      // or there are multiple fragments for a single pixel
      mag_filter: FilterMode::Linear,
      min_filter: FilterMode::Nearest,
      ..Default::default()
    });

    TextureResource { texture, view, sampler }
  }
}

