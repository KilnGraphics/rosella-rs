use rosella_rs::rosella::Rosella;
use rosella_rs::shader::vertex::{data_type, VertexFormat, VertexFormatBuilder, VertexFormatElement};

mod test_common;

#[test]
fn vertex_formats() {
    let position_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .build();

    let position_tex_color_normal_format = VertexFormatBuilder::new()
        .element(data_type::FLOAT, 3)
        .element(data_type::FLOAT, 2)
        .element(data_type::FLOAT, 3)
        .element(data_type::FLOAT, 3)
        .build();
}