use binrw::BinRead;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{fs, io::Cursor};
use syn::{
    Ident, LitStr, Result, Token, Visibility,
    parse::{Parse, ParseStream},
};

mod gfd;

struct IncludeShaderInput {
    vis: Visibility,
    name: Ident,
    _comma: Token![,],
    path: LitStr,
}

impl Parse for IncludeShaderInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = if input.peek(Token![pub]) {
            input.parse()?
        } else {
            Visibility::Inherited
        };
        let name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let path: LitStr = input.parse()?;
        Ok(Self {
            vis,
            name,
            _comma,
            path,
        })
    }
}

// fn pixel_shader_regs(regs: &cafe_sys::gx2::shader::PixelShaderRegisters) -> TokenStream {
//     let sq_pgm_resources_ps = {
//         let value = regs.sq_pgm_resources_ps.into_bits();
//         quote! { shader::registers::SQ_PGM_RESOURCES_PS::from_bits(#value) }
//     };

//     let sq_pgm_exports_ps = {
//         let value = regs.sq_pgm_exports_ps.into_bits();
//         quote! { shader::registers::SQ_PGM_EXPORTS_PS::from_bits(#value) }
//     };

//     let spi_ps_in_control_0 = {
//         let value = regs.spi_ps_in_control_0.into_bits();
//         quote! { shader::registers::SPI_PS_IN_CONTROL_0::from_bits(#value) }
//     };

//     let spi_ps_in_control_1 = {
//         let value = regs.spi_ps_in_control_1.into_bits();
//         quote! { shader::registers::SPI_PS_IN_CONTROL_1::from_bits(#value) }
//     };

//     let num_spi_ps_input_cntl = regs.num_spi_ps_input_cntl;

//     let spi_ps_input_cntls = regs.spi_ps_input_cntls.iter().map(|cntl| {
//         let value = cntl.into_bits();
//         quote! { shader::registers::SPI_PS_INPUT_CNTL::from_bits(#value) }
//     });

//     let cb_shader_mask = {
//         let value = regs.cb_shader_mask.into_bits();
//         quote! { shader::registers::CB_SHADER_MASK::from_bits(#value) }
//     };

//     let cb_shader_control = {
//         let value = regs.cb_shader_control.0;
//         quote! { shader::registers::CB_SHADER_CONTROL(#value) }
//     };

//     let db_shader_control = {
//         let value = regs.db_shader_control.into_bits();
//         quote! { shader::registers::DB_SHADER_CONTROL::from_bits(#value) }
//     };

//     let spi_input_z = {
//         let value = regs.spi_input_z.into_bits();
//         quote! { shader::registers::SPI_INPUT_Z::from_bits(#value) }
//     };

//     TokenStream::from(quote! {
//         shader::PixelShaderRegisters {
//             sq_pgm_resources_ps: #sq_pgm_resources_ps,
//             sq_pgm_exports_ps: #sq_pgm_exports_ps,
//             spi_ps_in_control_0: #spi_ps_in_control_0,
//             spi_ps_in_control_1: #spi_ps_in_control_1,
//             num_spi_ps_input_cntl: #num_spi_ps_input_cntl,
//             spi_ps_input_cntls: [#(#spi_ps_input_cntls),*],
//             cb_shader_mask: #cb_shader_mask,
//             cb_shader_control: #cb_shader_control,
//             db_shader_control: #db_shader_control,
//             spi_input_z: #spi_input_z,
//         }
//     })
// }

// fn pixel_shader(ps: &cafe_sys::gx2::shader::PixelShader, program: &[u8]) -> TokenStream {
//     let regs = proc_macro2::TokenStream::from(pixel_shader_regs(&ps.regs));

//     let shader_size = ps.shader_size;
//     let shader_ptr = {
//         let len = program.len();
//         let bytes = program.iter().map(|b| quote! { #b }).collect::<Vec<_>>();
//         quote! {
//             {
//                 static PROGRAM: Program<#len> = Program([#(#bytes),*]);
//                 PROGRAM.0.as_ptr().cast()
//             }
//         }
//     };
//     let shader_mode = match ps.shader_mode {
//         cafe_sys::gx2::shader::ShaderMode::UniformRegister => {
//             quote! { shader::ShaderMode::UniformRegister }
//         }
//         cafe_sys::gx2::shader::ShaderMode::UniformBlock => {
//             quote! { shader::ShaderMode::UniformBlock }
//         }
//         cafe_sys::gx2::shader::ShaderMode::Geometry => quote! { shader::ShaderMode::Geometry },
//         cafe_sys::gx2::shader::ShaderMode::Compute => quote! { shader::ShaderMode::Compute },
//     };
//     let num_uniform_blocks = ps.num_uniform_blocks;
//     let uniform_blocks = quote! { [].as_ptr() };
//     let num_uniforms = ps.num_uniforms;
//     let uniform_vars = quote! { [].as_ptr() };
//     let num_initial_values = ps.num_initial_values;
//     let initial_values = quote! { [].as_ptr() };
//     let num_loops = ps.num_loops;
//     let loop_vars = quote! { [].as_ptr() };
//     let num_samplers = ps.num_samplers;
//     let sampler_vars = quote! { [].as_ptr() };
//     let program = quote! { unsafe { std::mem::MaybeUninit::zeroed().assume_init() } };

//     TokenStream::from(quote! {
//         shader::PixelShader {
//             regs: #regs,
//             shader_size: #shader_size,
//             shader_ptr: #shader_ptr,
//             shader_mode: #shader_mode,
//             num_uniform_blocks: #num_uniform_blocks,
//             uniform_blocks: #uniform_blocks,
//             num_uniforms: #num_uniforms,
//             uniform_vars: #uniform_vars,
//             num_initial_values: #num_initial_values,
//             initial_values: #initial_values,
//             num_loops: #num_loops,
//             loop_vars: #loop_vars,
//             num_samplers: #num_samplers,
//             sampler_vars: #sampler_vars,
//             program: #program,
//         }
//     })
// }

fn vertex_shader(vs: &gfd::VertexShader, data: &[u8], program: &[u8]) -> proc_macro2::TokenStream {
    let regs = vs.regs();
    let shader_size = vs.shader_size();
    let shader_ptr = vs.shader_ptr(program);
    let shader_mode = vs.shader_mode();
    let num_uniform_blocks = vs.num_uniform_blocks();
    let uniform_blocks = vs.uniform_blocks(&data);
    let num_uniforms = vs.num_uniforms();
    let uniform_vars = vs.uniform_vars(&data);
    let num_initial_values = vs.num_initial_values();
    let initial_values = vs.initial_values(&data);
    let num_loops = vs.num_loops();
    let loop_vars = vs.loop_vars(&data);
    let num_samplers = vs.num_samplers();
    let sampler_vars = vs.sampler_vars(&data);
    let num_attribs = vs.num_attribs();
    let attrib_vars = vs.attrib_vars(&data);
    let ring_itemsize = vs.ring_itemsize();
    let has_stream_output = vs.has_stream_output();
    let stream_out_vertex_stride = vs.stream_out_vertex_stride();
    let program = vs.program();

    quote! {
        shader::VertexShader {
            regs: #regs,
            shader_size: #shader_size,
            shader_ptr: #shader_ptr,
            shader_mode: #shader_mode,
            num_uniform_blocks: #num_uniform_blocks,
            uniform_blocks: #uniform_blocks,
            num_uniforms: #num_uniforms,
            uniform_vars: #uniform_vars,
            num_initial_values: #num_initial_values,
            initial_values: #initial_values,
            num_loops: #num_loops,
            loop_vars: #loop_vars,
            num_samplers: #num_samplers,
            sampler_vars: #sampler_vars,
            num_attribs: #num_attribs,
            attrib_vars: #attrib_vars,
            ring_itemsize: #ring_itemsize,
            has_stream_output: #has_stream_output,
            stream_out_vertex_stride: #stream_out_vertex_stride,
            program: #program,
        }
    }
}

#[proc_macro]
pub fn include_shader(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as IncludeShaderInput);
    let vis = input.vis;
    let name = input.name;
    let path = input.path;
    let module_ident = format_ident!("__internal_{}", name);

    let mut file =
        Cursor::new(fs::read(path.value()).expect("Provided path does not point to a valid file"));
    let gfd = gfd::Gfd::read(&mut file)
        .expect("Provided path does contain an invalid / unsupported file");

    let todo = quote! { unsafe { ::core::mem::MaybeUninit::zeroed().assume_init() } };

    let mut vertex_shaders = vec![];
    let mut pixel_shaders = vec![];

    let mut iter = gfd.blocks.iter();
    while let Some(block) = iter.next() {
        match block.r#type {
            gfd::Type::VertexShaderHeader => {
                let header = block;
                let program = iter
                    .next()
                    .expect("VertexShaderProgram has to follow a VertexShaderHeader");
                assert_eq!(program.r#type, gfd::Type::VertexShaderProgram);

                //

                let shader = header.vertex_shader().unwrap();

                vertex_shaders.push(vertex_shader(&shader, &header.data, &program.data));
            }
            gfd::Type::PixelShaderHeader => {
                let header = block;
                let program = iter
                    .next()
                    .expect("PixelShaderProgram has to follow a PixelShaderHeader");
                assert_eq!(program.r#type, gfd::Type::PixelShaderProgram);

                //

                pixel_shaders.push(todo.clone());
            }
            _ => continue,
        }
    }

    let vertex_shader_len = vertex_shaders.len();
    let pixel_shader_len = pixel_shaders.len();

    TokenStream::from(quote! {
        #[allow(non_snake_case)]
        mod #module_ident {
            use ::cafe_rs::sys::gx2::shader;

            pub struct Shader {
                pub vertex: [shader::VertexShader; #vertex_shader_len],
                pub pixel: [shader::PixelShader; #pixel_shader_len],
            }
            unsafe impl Sync for Shader {}

            #[repr(C, align(256))]
            struct Program<const N: usize>([u8; N]);

            pub static SHADER: Shader = Shader {
                vertex: [#(#vertex_shaders),*],
                pixel: [#(#pixel_shaders),*],
            };
        }

        #vis const #name: &#module_ident::Shader = &#module_ident::SHADER;
    })
}
