use fallen_leaves_world::{world_gen_pipeline, img_gen};
fn main() {
    let pipeline = world_gen_pipeline::gen_world_pipeline_step_struct(1000, 1000, 500, 8, 0.7, 100.0);


    img_gen::gen_greyscale_img_from_vec(&pipeline.noise_base, format!("grey_pre_water_pass.png"));
    img_gen::gen_grey_with_waterlvl_highlighted(&pipeline.noise_base, 100.0, format!("normal.png"));
    img_gen::gen_grey_with_waterlvl_highlighted(&pipeline.smooth_noise, 100.0, format!("smooth.png"));
}
