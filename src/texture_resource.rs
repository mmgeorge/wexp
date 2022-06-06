use image::DynamicImage;
use wgpu::*;
use core::num;

pub struct TextureResource {
  pub texture: Texture,
  pub view : TextureView,
  pub sampler: Sampler,
}

impl TextureResource {

  pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: &str) -> Self {
    let img = image::load_from_memory(bytes).unwrap();

    Self::from_image(device, queue, &img, label)
  }

  // This approach brings in the image library which is a huge dependency. Currently we are missing
  // the WGPU method `queue.copyExternalImageToTexture` which would allow use to use ImageBitmaps directly
  // and handle this all in the browser
  pub fn from_image(device: &Device, queue: &Queue, image: &DynamicImage, label: &str) -> Self {
    let rgba = image.to_rgba8(); 
    let width = image.width();
    let height = image.height(); 
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

  // copyExternalImageToTexture not implemented
  // https://github.com/toji/webgpu-best-practices/blob/main/img-textures.md#creating-a-texture-from-an-image-url
  // pub async fn from_url(device: &Device, queue: &Queue, url: &str, label: &str) -> Result<Self, JsValue> {
  //   let mut request_init = RequestInit::new();

  //   request_init.method("GET");
  //   request_init.mode(RequestMode::Cors);

  //   let url = "./happy.png";
  //   let window = window().unwrap();

  //   let request =  Request::new_with_str_and_init(&url, &request_init)
  //     .map_err(|_| JsValue::from(String::from("Test")))?;
  
  //   let response_raw = JsFuture::from(window.fetch_with_request(&request)).await?;

  //   assert!(response_raw.is_instance_of::<Response>());

  //   let response: Response = response_raw.dyn_into()?; 
  //   let blob_future = JsFuture::from(response.array_buffer()?); 
  //   let blob = blob_future.await?;

  //   assert!(blob.is_instance_of::<ArrayBuffer>());

  //   let buf: ArrayBuffer = blob.dyn_into()?;
  //   let uint = Uint8Array::new(&buf).to_vec();
  //   // let b2 = web_sys::Blob::new_with_u8_array_sequence(&Uint8Array::new(&buf))?; 
  //   // let bitmap =  ImageBitmap::from(&b2);

  //   Ok(Self::from_image(device, queue, &uint, label))
  // }
  
  // pub fn from_image(device: &Device, queue: &Queue, buf: &[u8], label: &str) -> Self {
  //   let width = bitmap.width();
  //   let height = bitmap.height(); 
  //   let size = Extent3d { width, height, depth_or_array_layers: 1 };
  //   let texture = device.create_texture(&TextureDescriptor {
  //     label: Some(label),
  //     // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
  //     // COPY_DST means that we want to copy data to this texture
  //     usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
  //     size,
  //     dimension: TextureDimension::D2,
  //     format: TextureFormat::Rgba8UnormSrgb, 
  //     mip_level_count: 1,
  //     sample_count: 1,
  //   });

  //   let image_copy_texture = ImageCopyTexture {
  //       texture: &texture,
  //       mip_level: 0,
  //       origin: Origin3d::ZERO,
  //       aspect: TextureAspect::All
  //   }; 

  //   let layout = ImageDataLayout {
  //     offset: 0,
  //     bytes_per_row: num::NonZeroU32::new(4 * width),
  //     rows_per_image: num::NonZeroU32::new(height), 
  //   };

  //   // queue.copy_external_image_to_texture()

  //   let view = texture.create_view(&TextureViewDescriptor::default());
  //   let sampler = device.create_sampler(&SamplerDescriptor {
  //     label: Some(label), 
  //     // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
  //     // COPY_DST means that we want to copy data to this texture
  //     address_mode_u: AddressMode::ClampToEdge,
  //     address_mode_v: AddressMode::ClampToEdge,
  //     address_mode_w: AddressMode::ClampToEdge,
  //     // The mag_filter and min_filter options describe what to do when a fragment covers multiple pixels,
  //     // or there are multiple fragments for a single pixel
  //     mag_filter: FilterMode::Linear,
  //     min_filter: FilterMode::Nearest,
  //     ..Default::default()
  //   });

  //   TextureResource { texture, view, sampler }
  // }
}

