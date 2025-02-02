#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub file: &'static str,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    ///Names of uniformBuffers in the Shader. Cameras are excluded, because of their elevated
    ///role. They must be declared on a RenderScene level in the RessourceDescriptor. The @group of
    ///the buffers will correspond to their index here. Cameras will be appended, eg. start at
    ///index uniforms.len()
    pub uniforms: &'static [&'static str],
}

