
#[derive(Default, Clone, Debug)]
pub struct Talker
{
    pub name : String,
    pub sprite : String,
    pub sound : String,
    pub rate : Option<f32>,
}