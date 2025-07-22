use wgpu::BindGroupLayout;

pub struct BindGroups {
    binds: Vec<wgpu::BindGroupLayout>,
}

impl BindGroups {
    pub fn new() -> Self {
        Self {
            binds: vec![],
        }
    }

    pub fn add_bind(&mut self, bind_group: BindGroupLayout, ) {
        self.binds.push(bind_group);
    }

    pub fn collect_slice<'a>(&self) -> Vec<&BindGroupLayout> {
        let binding = &self.binds;
        let layout: Vec<&wgpu::BindGroupLayout> = binding.iter().collect();
        return layout;
    }
}