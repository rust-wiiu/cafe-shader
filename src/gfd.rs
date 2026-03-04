use binrw::{BinRead, binrw, helpers::until, meta::ReadEndian};
use proc_macro2::TokenStream;
use quote::quote;
use std::{fs, io::Cursor, mem::size_of};

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    EndOfFile = 0x01,
    Padding = 0x02,
    VertexShaderHeader = 0x03,
    VertexShaderProgram = 0x05,
    PixelShaderHeader = 0x06,
    PixelShaderProgram = 0x07,
    GeometryShaderHeader = 0x08,
    GeometryShaderProgram = 0x09,
    GeometryShaderCopyProgram = 0x010,
    TextureHeader = 0x011,
    TextureImageData = 0x012,
    TextureMipmapData = 0x013,
    ComputeShaderHeader = 0x014,
    ComputeShaderProgram = 0x015,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct Header {
    #[br(assert(magic == *b"Gfx2"))]
    pub magic: [u8; 4],
    #[br(assert(size == size_of::<Header>() as u32))]
    pub size: u32,
    #[br(assert(version == (7, 1)))]
    pub version: (u32, u32),
    #[br(assert(gpu == 2))]
    pub gpu: u32,
    #[br(assert(align == 0 || align == 1))]
    pub align: u32,
    pub reserved1: u32,
    pub reserved2: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct Block {
    #[br(assert(magic == *b"BLK{"))]
    pub magic: [u8; 4],
    #[br(assert(size == size_of::<Block>() as u32 - size_of::<Vec<u8>>() as u32))]
    pub size: u32,
    #[br(assert(version == (1, 0)))]
    pub version: (u32, u32),
    pub r#type: Type,
    pub len: u32,
    pub id: u32,
    pub index: u32,
    #[br(count = len)]
    pub data: Vec<u8>,
}

impl Block {
    pub const TEXT: u32 = 0xCA70_0000;
    pub const DATA: u32 = 0xD060_0000;

    pub fn vertex_shader(&self) -> Option<VertexShader> {
        if self.r#type != Type::VertexShaderHeader {
            None
        } else {
            Some(
                VertexShader::read(&mut Cursor::new(&self.data[..(size_of::<VertexShader>())]))
                    .unwrap(),
            )
        }
    }

    pub fn pixel_shader(&self) -> Option<PixelShader> {
        if self.r#type != Type::PixelShaderHeader {
            None
        } else {
            Some(
                PixelShader::read(&mut Cursor::new(&self.data[..(size_of::<PixelShader>())]))
                    .unwrap(),
            )
        }
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct RelocationInfo {
    #[br(assert(magic == *b"}BLK"))]
    pub magic: [u8; 4],
    #[br(assert(size == size_of::<RelocationInfo>() as u32))]
    pub size: u32,
    #[br(assert(zero == 0))]
    pub zero: u32,
    pub data_size: u32,
    #[br(assert(data_offset & 0xFFF0_0000 == 0xD060_0000))]
    pub data_offset: u32,
    pub text_size: u32,
    #[br(assert(text_offset & 0xFFF0_0000 == 0xD060_0000))]
    pub text_offset: u32,
    #[br(assert(relo_base == 0))]
    pub relo_base: u32,
    pub relo_count: u32,
    #[br(assert(relo_offset & 0xFFF0_0000 == 0xD060_0000))]
    pub relo_offset: u32,
}

#[binrw]
#[br(big)]
pub struct Gfd {
    pub header: Header,
    #[br(parse_with = until(|b: &Block| matches!(b.r#type, Type::EndOfFile)))]
    pub blocks: Vec<Block>,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct VertexShader {
    pub regs: [u32; 52],
    pub shader_size: u32,
    pub shader_ptr: u32,
    pub shader_mode: u32,
    pub num_uniform_blocks: u32,
    pub uniform_blocks: u32,
    pub num_uniforms: u32,
    pub uniform_vars: u32,
    pub num_initial_values: u32,
    pub initial_values: u32,
    pub num_loops: u32,
    pub loop_vars: u32,
    pub num_samplers: u32,
    pub sampler_vars: u32,
    pub num_attribs: u32,
    pub attrib_vars: u32,
    pub ring_itemsize: u32,
    pub has_stream_output: u32,
    pub stream_out_vertex_stride: [u32; 4],
    pub program: [u32; 4],
}

impl VertexShader {
    pub fn regs(&self) -> TokenStream {
        let sq_pgm_resources_vs = {
            let value = self.regs[0];
            quote! { shader::registers::SQ_PGM_RESOURCES_VS::from_bits(#value) }
        };

        let vgt_primitiveid_en = {
            let value = self.regs[1];
            quote! { shader::registers::VGT_PRIMITIVEID_EN::from_bits(#value) }
        };

        let spi_vs_out_config = {
            let value = self.regs[2];
            quote! { shader::registers::SPI_VS_OUT_CONFIG::from_bits(#value) }
        };

        let num_spi_vs_out_id = self.regs[3];

        let spi_vs_out_id = self.regs[4..14].iter().map(|&reg| {
            quote! { shader::registers::SPI_VS_OUT_ID::from_bits(#reg) }
        });

        let pa_cl_vs_out_cntl = {
            let value = self.regs[14];
            quote! { shader::registers::PA_CL_VS_OUT_CNTL::from_bits(#value) }
        };

        let sq_vtx_semantic_clear = {
            let value = self.regs[15];
            quote! { shader::registers::SQ_VTX_SEMANTIC_CLEAR(#value) }
        };

        let num_sq_vtx_semantic = self.regs[16];

        let sq_vtx_semantic = self.regs[17..49].iter().map(|&reg| {
            quote! { shader::registers::SQ_VTX_SEMANTIC::from_bits(#reg) }
        });

        let vgt_strmout_buffer_en = {
            let value = self.regs[49];
            quote! { shader::registers::VGT_STRMOUT_BUFFER_EN::from_bits(#value) }
        };

        let vgt_vertex_reuse_block_cntl = {
            let value = self.regs[50];
            quote! { shader::registers::VGT_VERTEX_REUSE_BLOCK_CNTL::from_bits(#value) }
        };

        let vgt_hos_reuse_depth = {
            let value = self.regs[51];
            quote! { shader::registers::VGT_HOS_REUSE_DEPTH::from_bits(#value) }
        };

        quote! {
            shader::VertexShaderRegisters {
                sq_pgm_resources_vs: #sq_pgm_resources_vs,
                vgt_primitiveid_en: #vgt_primitiveid_en,
                spi_vs_out_config: #spi_vs_out_config,
                num_spi_vs_out_id: #num_spi_vs_out_id,
                spi_vs_out_id: [#(#spi_vs_out_id),*],
                pa_cl_vs_out_cntl: #pa_cl_vs_out_cntl,
                sq_vtx_semantic_clear: #sq_vtx_semantic_clear,
                num_sq_vtx_semantic: #num_sq_vtx_semantic,
                sq_vtx_semantic: [#(#sq_vtx_semantic),*],
                vgt_strmout_buffer_en: #vgt_strmout_buffer_en,
                vgt_vertex_reuse_block_cntl: #vgt_vertex_reuse_block_cntl,
                vgt_hos_reuse_depth: #vgt_hos_reuse_depth,
            }
        }
    }

    pub fn shader_size(&self) -> TokenStream {
        let value = self.shader_size;
        quote! {#value }
    }

    pub fn shader_ptr(&self, program: &[u8]) -> TokenStream {
        let len = program.len();
        let bytes = program.iter().map(|b| quote! { #b }).collect::<Vec<_>>();
        quote! {
            {
                static PROGRAM: Program<#len> = Program([#(#bytes),*]);
                PROGRAM.0.as_ptr().cast()
            }
        }
    }

    pub fn shader_mode(&self) -> TokenStream {
        match self.shader_mode {
            0 => {
                quote! { shader::ShaderMode::UniformRegister }
            }
            1 => {
                quote! { shader::ShaderMode::UniformBlock }
            }
            2 => quote! { shader::ShaderMode::Geometry },
            3 => quote! { shader::ShaderMode::Compute },
            _ => panic!("Invalid shader mode"),
        }
    }

    pub fn num_uniform_blocks(&self) -> TokenStream {
        let value = self.num_uniform_blocks;
        quote! {#value }
    }

    pub fn uniform_blocks(&self, data: &[u8]) -> TokenStream {
        if self.uniform_blocks == 0 {
            quote! { ::core::ptr::null() }
        } else {
            let offset = (self.uniform_blocks & !Block::DATA) as usize;
            let mut reader = Cursor::new(&data[offset..]);
            let mut structs = vec![];

            for _ in 0..self.num_uniform_blocks {
                structs.push(UniformBlock::read(&mut reader).unwrap());
            }

            let vec = structs
                .iter()
                .map(|s| {
                    let name = if s.name == 0 {
                        quote! { ::core::ptr::null() }
                    } else {
                        let offset = (s.name & !Block::TEXT) as usize;
                        let bytes = &data[offset..]
                            .iter()
                            .take_while(|&b| *b != 0)
                            .cloned()
                            .collect::<Vec<_>>();

                        let str = String::from_utf8_lossy(&bytes);

                        quote! { c"#str".as_ptr() }
                    };
                    let location = s.location;
                    let size = s.size;

                    quote! {
                        shader::UniformBlock {
                            name: #name,
                            location: #location,
                            size: #size,
                        }
                    }
                })
                .collect::<Vec<TokenStream>>();

            quote! {}
        }
    }

    pub fn num_uniforms(&self) -> TokenStream {
        let value = self.num_uniforms;
        quote! { #value }
    }

    pub fn uniform_vars(&self, data: &[u8]) -> TokenStream {
        quote! { ::core::ptr::null() }
    }

    pub fn num_initial_values(&self) -> TokenStream {
        let value = self.num_initial_values;
        quote! { #value }
    }

    pub fn initial_values(&self, data: &[u8]) -> TokenStream {
        quote! { ::core::ptr::null() }
    }

    pub fn num_loops(&self) -> TokenStream {
        let value = self.num_loops;
        quote! { #value }
    }

    pub fn loop_vars(&self, data: &[u8]) -> TokenStream {
        quote! { ::core::ptr::null() }
    }

    pub fn num_samplers(&self) -> TokenStream {
        let value = self.num_samplers;
        quote! { #value }
    }

    pub fn sampler_vars(&self, data: &[u8]) -> TokenStream {
        quote! { ::core::ptr::null() }
    }

    pub fn num_attribs(&self) -> TokenStream {
        let value = self.num_attribs;
        quote! { #value }
    }

    pub fn attrib_vars(&self, data: &[u8]) -> TokenStream {
        if self.num_attribs == 0 {
            quote! { ::core::ptr::null() }
        } else {
            let offset = (self.attrib_vars & !Block::DATA) as usize;
            let mut reader = Cursor::new(&data[offset..]);
            let mut structs = vec![];

            for _ in 0..self.num_attribs {
                structs.push(AttribVar::read(&mut reader).unwrap());
            }

            let vec = structs
                .iter()
                .map(|s| {
                    let name = if s.name == 0 {
                        quote! { ::core::ptr::null() }
                    } else {
                        let offset = (s.name & !Block::TEXT) as usize;
                        let bytes = &data[offset..]
                            .iter()
                            .take_while(|&b| *b != 0)
                            .cloned()
                            .collect::<Vec<_>>();

                        let str = String::from_utf8_lossy(&bytes);
                        let str = syn::LitCStr::new(
                            std::ffi::CString::new(str.into_owned()).unwrap().as_c_str(),
                            proc_macro2::Span::call_site(),
                        );

                        quote! { #str.as_ptr() }
                    };

                    let r#type = {
                        let value = s.r#type;
                        quote! { unsafe { ::core::mem::transmute( #value ) } }
                    };
                    let array_count = s.array_count;
                    let location = s.location;

                    quote! {
                        shader::AttribVar {
                            name: #name,
                            r#type: #r#type,
                            array_count: #array_count,
                            location: #location,
                        }
                    }
                })
                .collect::<Vec<TokenStream>>();

            quote! { [#(#vec),*].as_ptr() }
        }
    }

    pub fn ring_itemsize(&self) -> TokenStream {
        let value = self.ring_itemsize;
        quote! { #value }
    }

    pub fn has_stream_output(&self) -> TokenStream {
        let value = self.has_stream_output as i32;
        quote! { #value }
    }

    pub fn stream_out_vertex_stride(&self) -> TokenStream {
        let values = self.stream_out_vertex_stride;
        quote! { [#(#values),*] }
    }

    pub fn program(&self) -> TokenStream {
        quote! { unsafe { ::core::mem::MaybeUninit::zeroed().assume_init() } }
    }
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct PixelShader {
    pub regs: [u32; 41],
    pub shader_size: u32,
    pub shader_ptr: u32,
    pub shader_mode: u32,
    pub num_uniform_blocks: u32,
    pub uniform_blocks: u32,
    pub num_uniforms: u32,
    pub uniform_vars: u32,
    pub num_initial_values: u32,
    pub initial_values: u32,
    pub num_loops: u32,
    pub loop_vars: u32,
    pub num_samplers: u32,
    pub sampler_vars: u32,
    pub program: [u32; 4],
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct UniformBlock {
    pub name: u32,
    pub location: u32,
    pub size: u32,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct UniformVar {
    pub name: u32,
    pub r#type: u32,
    pub array_count: u32,
    pub offset: u32,
    pub block_index: u32,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct UniformInitialValue {
    pub value: [f32; 4],
    pub offset: u32,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct LoopVar {
    pub offset: u32,
    pub value: u32,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct SamplerVar {
    pub name: u32,
    pub r#type: u32,
    pub location: u32,
}

#[binrw]
#[br(big)]
#[repr(C)]
#[derive(Debug)]
pub struct AttribVar {
    pub name: u32,
    pub r#type: u32,
    pub array_count: u32,
    pub location: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shader() -> Cursor<Vec<u8>> {
        Cursor::new(fs::read("tests/shader.gsh").unwrap())
    }

    #[test]
    fn header() {
        let mut file = shader();
        let _ = Header::read(&mut file).unwrap();
    }

    #[test]
    fn gfd() {
        let mut file = shader();
        let gfd = Gfd::read(&mut file).unwrap();

        assert!(gfd.blocks.len() == 5);

        assert!(gfd.blocks[0].r#type == Type::VertexShaderHeader);
        assert!(gfd.blocks[1].r#type == Type::VertexShaderProgram);
        assert!(gfd.blocks[2].r#type == Type::PixelShaderHeader);
        assert!(gfd.blocks[3].r#type == Type::PixelShaderProgram);
        assert!(gfd.blocks[4].r#type == Type::EndOfFile);
    }

    #[test]
    fn vertex_shader() {
        let mut file = shader();
        let gfd = Gfd::read(&mut file).unwrap();

        let data = &gfd.blocks[0].data;
        let program = &gfd.blocks[1].data;

        let relo = RelocationInfo::read(&mut Cursor::new(
            &data[(data.len() - size_of::<RelocationInfo>())..],
        ))
        .unwrap();

        let shader = gfd.blocks[0].vertex_shader().unwrap();

        assert_eq!(shader.shader_size, program.len() as u32);

        println!("{:X?}", &shader);

        let patch_table = {
            let start = (relo.relo_offset & 0x000F_FFFF) as usize;
            let end = start + (relo.relo_count * 4) as usize;
            &data[start..end]
        };

        let mut offsets = vec![];
        for chunk in patch_table.chunks_exact(4) {
            let offset = u32::from_be_bytes(chunk.try_into().unwrap());

            if offset == 0 {
                continue;
            }

            offsets.push(offset);
        }

        println!("Offsets: {:X?}", offsets);

        println!("{:X?}", relo);
    }

    #[test]
    fn pixel_shader() {
        let mut file = shader();
        let gfd = Gfd::read(&mut file).unwrap();

        let data = &gfd.blocks[0].data;
        let program = &gfd.blocks[1].data;

        let shader = gfd.blocks[0].vertex_shader().unwrap();

        assert_eq!(shader.shader_size, program.len() as u32);
    }
}
