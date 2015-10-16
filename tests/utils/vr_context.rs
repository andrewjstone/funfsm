pub trait VRContext: Send + 'static {

}

pub struct VRContextImp;

impl VRContext for VRContextImp {

}

impl VRContextImp {
    pub fn new() -> Self {
        VRContextImp
    }
}
