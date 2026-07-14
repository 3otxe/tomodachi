

use tomodachi_shared::Mood;

pub const SPRITE_SIZE: u32 = 32;

pub const SCALE: u32 = 4;

pub const WINDOW_SIZE: u32 = SPRITE_SIZE * SCALE;

#[allow(dead_code)]
mod colors {
    pub const TRANSPARENT: u32 = 0x00000000;
    
    pub const OUTLINE: u32 = 0xFF2B2D42; 
    pub const BODY_LIGHT: u32 = 0xFF8ECAE6; 
    pub const BODY_CORE: u32 = 0xFF219EBC; 
    pub const BODY_SHADOW: u32 = 0xFF126782; 
    
    pub const EYE_WHITE: u32 = 0xFFFFFFFF;
    pub const EYE_PUPIL: u32 = 0xFF023047;
    pub const BLUSH: u32 = 0xFFFFAFCC; 
    pub const MOUTH: u32 = 0xFFFB8500; 
    
    pub const SWEAT: u32 = 0xFFE0FBFC; 
    pub const ZZZ: u32 = 0xFF8D99AE; 
    pub const ANGER: u32 = 0xFFD62828; 
    pub const ANGER_VEIN: u32 = 0xFF9B2226; 
    pub const SUNGLASSES: u32 = 0xFF000000; 
    pub const RAIN: u32 = 0xFF4A4E69; 
    pub const SPARKLE: u32 = 0xFFFFB703; 
}

fn set_pixel(canvas: &mut [u32], x: i32, y: i32, color: u32) {
    if x >= 0 && x < SPRITE_SIZE as i32 && y >= 0 && y < SPRITE_SIZE as i32 {
        canvas[(y * SPRITE_SIZE as i32 + x) as usize] = color;
    }
}

fn hline(canvas: &mut [u32], x1: i32, x2: i32, y: i32, color: u32) {
    for x in x1..=x2 {
        set_pixel(canvas, x, y, color);
    }
}

fn vline(canvas: &mut [u32], x: i32, y1: i32, y2: i32, color: u32) {
    for y in y1..=y2 {
        set_pixel(canvas, x, y, color);
    }
}

pub fn render_sprite(mood: Mood, tick: u32) -> Vec<u32> {
    let mut canvas = vec![colors::TRANSPARENT; (SPRITE_SIZE * SPRITE_SIZE) as usize];

    let is_angry = mood == Mood::Furious;
    let y_bounce = if mood == Mood::Happy && tick % 4 < 2 { 1 } else { 0 };
    
    draw_body(&mut canvas, 16, 17 - y_bounce, is_angry);

    match mood {
        Mood::Idle => draw_idle(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Happy => draw_happy(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Smug => draw_smug(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Sad => draw_sad(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Nervous => draw_nervous(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Furious => draw_furious(&mut canvas, tick, 16, 17 - y_bounce),
        Mood::Sleeping => draw_sleeping(&mut canvas, tick, 16, 17 - y_bounce),
    }

    scale_up(&canvas)
}

fn draw_body(canvas: &mut [u32], cx: i32, cy: i32, is_angry: bool) {
    
    let base = if is_angry { colors::ANGER } else { colors::BODY_CORE };
    let light = if is_angry { colors::ANGER } else { colors::BODY_LIGHT };
    let shadow = if is_angry { colors::ANGER_VEIN } else { colors::BODY_SHADOW };
    
    hline(canvas, cx-4, cx+3, cy-10, colors::OUTLINE);
    hline(canvas, cx-7, cx-5, cy-9, colors::OUTLINE);
    hline(canvas, cx+4, cx+6, cy-9, colors::OUTLINE);
    set_pixel(canvas, cx-8, cy-8, colors::OUTLINE);
    set_pixel(canvas, cx+7, cy-8, colors::OUTLINE);
    set_pixel(canvas, cx-9, cy-7, colors::OUTLINE);
    set_pixel(canvas, cx+8, cy-7, colors::OUTLINE);
    
    vline(canvas, cx-10, cy-6, cy+3, colors::OUTLINE);
    vline(canvas, cx+9, cy-6, cy+3, colors::OUTLINE);
    
    set_pixel(canvas, cx-9, cy+4, colors::OUTLINE);
    set_pixel(canvas, cx+8, cy+4, colors::OUTLINE);
    set_pixel(canvas, cx-8, cy+5, colors::OUTLINE);
    set_pixel(canvas, cx+7, cy+5, colors::OUTLINE);
    set_pixel(canvas, cx-7, cy+6, colors::OUTLINE);
    set_pixel(canvas, cx+6, cy+6, colors::OUTLINE);
    
    hline(canvas, cx-6, cx-3, cy+7, colors::OUTLINE);
    hline(canvas, cx+2, cx+5, cy+7, colors::OUTLINE);
    hline(canvas, cx-2, cx+1, cy+6, colors::OUTLINE);
    
    hline(canvas, cx-4, cx+3, cy-9, light);
    hline(canvas, cx-7, cx+6, cy-8, light);
    hline(canvas, cx-8, cx+7, cy-7, light);
    hline(canvas, cx-9, cx+8, cy-6, light);
    
    hline(canvas, cx-9, cx+8, cy-5, base);
    hline(canvas, cx-9, cx+8, cy-4, base);
    hline(canvas, cx-9, cx+8, cy-3, base);
    hline(canvas, cx-9, cx+8, cy-2, base);
    hline(canvas, cx-9, cx+8, cy-1, base);
    hline(canvas, cx-9, cx+8, cy, base);
    hline(canvas, cx-9, cx+8, cy+1, base);
    
    hline(canvas, cx-9, cx+8, cy+2, shadow);
    hline(canvas, cx-9, cx+8, cy+3, shadow);
    hline(canvas, cx-8, cx+7, cy+4, shadow);
    hline(canvas, cx-7, cx+6, cy+5, shadow);
    
    hline(canvas, cx-6, cx-3, cy+6, shadow);
    
    hline(canvas, cx+2, cx+5, cy+6, shadow);
}

fn draw_eyes_open(canvas: &mut [u32], cx: i32, cy: i32) {
    
    hline(canvas, cx-5, cx-3, cy-4, colors::EYE_WHITE);
    hline(canvas, cx-5, cx-3, cy-3, colors::EYE_WHITE);
    hline(canvas, cx-5, cx-3, cy-2, colors::EYE_WHITE);
    
    set_pixel(canvas, cx-4, cy-3, colors::EYE_PUPIL);
    set_pixel(canvas, cx-4, cy-2, colors::EYE_PUPIL);
    
    hline(canvas, cx+2, cx+4, cy-4, colors::EYE_WHITE);
    hline(canvas, cx+2, cx+4, cy-3, colors::EYE_WHITE);
    hline(canvas, cx+2, cx+4, cy-2, colors::EYE_WHITE);
    
    set_pixel(canvas, cx+3, cy-3, colors::EYE_PUPIL);
    set_pixel(canvas, cx+3, cy-2, colors::EYE_PUPIL);
}

fn draw_eyes_closed(canvas: &mut [u32], cx: i32, cy: i32) {
    hline(canvas, cx-5, cx-3, cy-2, colors::OUTLINE);
    hline(canvas, cx+2, cx+4, cy-2, colors::OUTLINE);
}

fn draw_eyes_happy(canvas: &mut [u32], cx: i32, cy: i32) {
    
    set_pixel(canvas, cx-5, cy-2, colors::OUTLINE);
    set_pixel(canvas, cx-4, cy-3, colors::OUTLINE);
    set_pixel(canvas, cx-3, cy-2, colors::OUTLINE);
    
    set_pixel(canvas, cx+2, cy-2, colors::OUTLINE);
    set_pixel(canvas, cx+3, cy-3, colors::OUTLINE);
    set_pixel(canvas, cx+4, cy-2, colors::OUTLINE);
}

fn draw_eyes_sad(canvas: &mut [u32], cx: i32, cy: i32) {
    
    hline(canvas, cx-5, cx-3, cy-4, colors::EYE_WHITE);
    hline(canvas, cx-5, cx-3, cy-3, colors::EYE_WHITE);
    hline(canvas, cx-5, cx-3, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, cx-3, cy-3, colors::EYE_PUPIL); 
    set_pixel(canvas, cx-3, cy-2, colors::EYE_PUPIL);
    
    hline(canvas, cx+2, cx+4, cy-4, colors::EYE_WHITE);
    hline(canvas, cx+2, cx+4, cy-3, colors::EYE_WHITE);
    hline(canvas, cx+2, cx+4, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, cx+2, cy-3, colors::EYE_PUPIL); 
    set_pixel(canvas, cx+2, cy-2, colors::EYE_PUPIL);
    
    hline(canvas, cx-5, cx-3, cy-4, colors::BODY_LIGHT);
    set_pixel(canvas, cx-3, cy-4, colors::EYE_WHITE);
    hline(canvas, cx+2, cx+4, cy-4, colors::BODY_LIGHT);
    set_pixel(canvas, cx+2, cy-4, colors::EYE_WHITE);
}

fn draw_blush(canvas: &mut [u32], cx: i32, cy: i32) {
    hline(canvas, cx-7, cx-5, cy, colors::BLUSH);
    hline(canvas, cx+4, cx+6, cy, colors::BLUSH);
}

fn draw_mouth_small(canvas: &mut [u32], cx: i32, cy: i32) {
    hline(canvas, cx-1, cx, cy+1, colors::OUTLINE);
}

fn draw_mouth_open(canvas: &mut [u32], cx: i32, cy: i32) {
    set_pixel(canvas, cx-1, cy+1, colors::OUTLINE);
    set_pixel(canvas, cx, cy+1, colors::OUTLINE);
    set_pixel(canvas, cx-1, cy+2, colors::MOUTH);
    set_pixel(canvas, cx, cy+2, colors::MOUTH);
    set_pixel(canvas, cx-1, cy+3, colors::OUTLINE);
    set_pixel(canvas, cx, cy+3, colors::OUTLINE);
    set_pixel(canvas, cx-2, cy+2, colors::OUTLINE);
    set_pixel(canvas, cx+1, cy+2, colors::OUTLINE);
}

fn draw_idle(canvas: &mut [u32], tick: u32, cx: i32, cy: i32) {
    
    if tick % 20 == 0 {
        draw_eyes_closed(canvas, cx, cy);
    } else {
        draw_eyes_open(canvas, cx, cy);
    }
    draw_blush(canvas, cx, cy);
    draw_mouth_small(canvas, cx, cy);
}

fn draw_happy(canvas: &mut [u32], tick: u32, cx: i32, cy: i32) {
    draw_eyes_happy(canvas, cx, cy);
    draw_blush(canvas, cx, cy);
    draw_mouth_open(canvas, cx, cy);
    
    if tick % 2 == 0 {
        set_pixel(canvas, cx-10, cy-10, colors::SPARKLE);
        set_pixel(canvas, cx-10, cy-12, colors::SPARKLE);
        set_pixel(canvas, cx-11, cy-11, colors::SPARKLE);
        set_pixel(canvas, cx-9, cy-11, colors::SPARKLE);
        
        set_pixel(canvas, cx+10, cy, colors::SPARKLE);
        set_pixel(canvas, cx+9, cy+1, colors::SPARKLE);
        set_pixel(canvas, cx+11, cy+1, colors::SPARKLE);
        set_pixel(canvas, cx+10, cy+2, colors::SPARKLE);
    }
}

fn draw_smug(canvas: &mut [u32], _tick: u32, cx: i32, cy: i32) {
    draw_mouth_small(canvas, cx, cy-1);
    
    hline(canvas, cx-8, cx+7, cy-4, colors::OUTLINE);
    
    hline(canvas, cx-7, cx-2, cy-3, colors::SUNGLASSES);
    hline(canvas, cx-6, cx-3, cy-2, colors::SUNGLASSES);
    
    hline(canvas, cx+1, cx+6, cy-3, colors::SUNGLASSES);
    hline(canvas, cx+2, cx+5, cy-2, colors::SUNGLASSES);
    
    hline(canvas, cx-1, cx, cy-3, colors::OUTLINE);
    
    set_pixel(canvas, cx-6, cy-3, colors::EYE_WHITE);
    set_pixel(canvas, cx+2, cy-3, colors::EYE_WHITE);
}

fn draw_sad(canvas: &mut [u32], tick: u32, cx: i32, cy: i32) {
    draw_eyes_sad(canvas, cx, cy);
    
    hline(canvas, cx-1, cx, cy+2, colors::OUTLINE);
    set_pixel(canvas, cx-2, cy+3, colors::OUTLINE);
    set_pixel(canvas, cx+1, cy+3, colors::OUTLINE);
    
    let offset1 = (tick % 8) as i32;
    let offset2 = ((tick + 4) % 8) as i32;
    
    set_pixel(canvas, cx-12, cy-8 + offset1, colors::RAIN);
    set_pixel(canvas, cx-12, cy-7 + offset1, colors::RAIN);
    
    set_pixel(canvas, cx+11, cy-4 + offset2, colors::RAIN);
    set_pixel(canvas, cx+11, cy-3 + offset2, colors::RAIN);
}

fn draw_nervous(canvas: &mut [u32], _tick: u32, cx: i32, cy: i32) {
    
    hline(canvas, cx-6, cx-2, cy-4, colors::EYE_WHITE);
    hline(canvas, cx-6, cx-2, cy-3, colors::EYE_WHITE);
    hline(canvas, cx-6, cx-2, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, cx-4, cy-3, colors::EYE_PUPIL);
    
    hline(canvas, cx+1, cx+5, cy-4, colors::EYE_WHITE);
    hline(canvas, cx+1, cx+5, cy-3, colors::EYE_WHITE);
    hline(canvas, cx+1, cx+5, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, cx+3, cy-3, colors::EYE_PUPIL);
    
    set_pixel(canvas, cx-2, cy+1, colors::OUTLINE);
    set_pixel(canvas, cx-1, cy+2, colors::OUTLINE);
    set_pixel(canvas, cx, cy+1, colors::OUTLINE);
    set_pixel(canvas, cx+1, cy+2, colors::OUTLINE);
    
    set_pixel(canvas, cx+8, cy-6, colors::SWEAT);
    hline(canvas, cx+7, cx+9, cy-5, colors::SWEAT);
    hline(canvas, cx+7, cx+9, cy-4, colors::SWEAT);
    set_pixel(canvas, cx+8, cy-3, colors::SWEAT);
}

fn draw_furious(canvas: &mut [u32], tick: u32, cx: i32, cy: i32) {
    
    let shake_x = if tick % 2 == 0 { 1 } else { -1 };
    let ncx = cx + shake_x;
    
    hline(canvas, ncx-5, ncx-3, cy-3, colors::EYE_WHITE);
    hline(canvas, ncx-5, ncx-3, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, ncx-4, cy-2, colors::EYE_PUPIL);
    
    set_pixel(canvas, ncx-6, cy-5, colors::OUTLINE);
    set_pixel(canvas, ncx-5, cy-4, colors::OUTLINE);
    set_pixel(canvas, ncx-4, cy-4, colors::OUTLINE);
    set_pixel(canvas, ncx-3, cy-3, colors::OUTLINE);
    
    hline(canvas, ncx+2, ncx+4, cy-3, colors::EYE_WHITE);
    hline(canvas, ncx+2, ncx+4, cy-2, colors::EYE_WHITE);
    set_pixel(canvas, ncx+3, cy-2, colors::EYE_PUPIL);
    
    set_pixel(canvas, ncx+5, cy-5, colors::OUTLINE);
    set_pixel(canvas, ncx+4, cy-4, colors::OUTLINE);
    set_pixel(canvas, ncx+3, cy-4, colors::OUTLINE);
    set_pixel(canvas, ncx+2, cy-3, colors::OUTLINE);
    
    hline(canvas, ncx-2, ncx+1, cy+1, colors::OUTLINE);
    set_pixel(canvas, ncx-3, cy+2, colors::OUTLINE);
    set_pixel(canvas, ncx+2, cy+2, colors::OUTLINE);
    
    set_pixel(canvas, ncx+6, cy-9, colors::ANGER_VEIN);
    set_pixel(canvas, ncx+7, cy-8, colors::ANGER_VEIN);
    set_pixel(canvas, ncx+5, cy-8, colors::ANGER_VEIN);
    set_pixel(canvas, ncx+6, cy-7, colors::ANGER_VEIN);
}

fn draw_sleeping(canvas: &mut [u32], tick: u32, cx: i32, cy: i32) {
    draw_eyes_closed(canvas, cx, cy);
    
    let bubble_size = (tick % 6) / 2;
    if bubble_size == 0 {
        set_pixel(canvas, cx+3, cy+1, colors::SWEAT);
    } else if bubble_size == 1 {
        hline(canvas, cx+2, cx+3, cy+1, colors::SWEAT);
        hline(canvas, cx+2, cx+3, cy+2, colors::SWEAT);
    } else {
        set_pixel(canvas, cx+3, cy, colors::SWEAT);
        hline(canvas, cx+2, cx+4, cy+1, colors::SWEAT);
        hline(canvas, cx+2, cx+4, cy+2, colors::SWEAT);
        set_pixel(canvas, cx+3, cy+3, colors::SWEAT);
        
        set_pixel(canvas, cx+4, cy+1, colors::EYE_WHITE);
    }
    
    let z_phase = tick % 12;
    if z_phase > 2 {
        
        hline(canvas, cx+8, cx+9, cy-8, colors::ZZZ);
        set_pixel(canvas, cx+8, cy-7, colors::ZZZ);
        hline(canvas, cx+8, cx+9, cy-6, colors::ZZZ);
    }
    if z_phase > 6 {
        
        hline(canvas, cx+10, cx+12, cy-12, colors::ZZZ);
        set_pixel(canvas, cx+11, cy-11, colors::ZZZ);
        set_pixel(canvas, cx+10, cy-10, colors::ZZZ);
        hline(canvas, cx+10, cx+12, cy-9, colors::ZZZ);
    }
}

fn scale_up(src: &[u32]) -> Vec<u32> {
    let mut dst = vec![colors::TRANSPARENT; (WINDOW_SIZE * WINDOW_SIZE) as usize];
    for y in 0..SPRITE_SIZE {
        for x in 0..SPRITE_SIZE {
            let px = src[(y * SPRITE_SIZE + x) as usize];
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let dst_x = x * SCALE + dx;
                    let dst_y = y * SCALE + dy;
                    dst[(dst_y * WINDOW_SIZE + dst_x) as usize] = px;
                }
            }
        }
    }
    dst
}
