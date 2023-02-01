pub mod type_tree;

use crate::def_unity_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObjectRef;
use binrw::{binrw, BinRead};
use std::{
    fmt,
    io::{Cursor, Read, Seek, SeekFrom},
};

use super::ClassIDType;

def_unity_class!(AnimationClip);

pub trait AnimationClipObject: fmt::Debug {}

#[binrw]
#[derive(Debug, Clone)]
pub struct StreamedFrame {
    pub time: f32,
    num_keys: i32,
    #[br(count(num_keys))]
    pub key_list: Vec<StreamedCurveKey>,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct StreamedCurveKey {
    pub index: i32,
    pub coeff: [f32; 4],
    #[brw(ignore)]
    pub in_slope: f32,
}

impl StreamedCurveKey {
    pub fn get_out_slope(&self) -> &f32 {
        &self.coeff[2]
    }
    pub fn get_value(&self) -> &f32 {
        &self.coeff[3]
    }

    pub fn calculate_next_in_slope(&self, dx: f32, rhs: &StreamedCurveKey) -> f32 {
        //Stepped
        if self.coeff[0] == 0.0 && self.coeff[1] == 0.0 && self.coeff[2] == 0.0 {
            return f32::INFINITY;
        }

        let dx = *std::cmp::max(
            ordered_float::OrderedFloat(dx),
            ordered_float::OrderedFloat(0.0001),
        );
        let dy = rhs.get_value() - self.get_value();
        let length = 1.0 / (dx * dx);
        let d1 = self.get_out_slope() * dx;
        let d2 = dy + dy + dy - d1 - d1 - self.coeff[1] / length;
        return d2 / dx;
    }
}

pub fn streamed_clip_read_data<R: Read + Seek>(
    reader: &mut R,
) -> anyhow::Result<Vec<StreamedFrame>> {
    let mut streamed_frames = Vec::new();
    let end_pos = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;
    while end_pos > reader.seek(SeekFrom::Current(0))? {
        streamed_frames.push(StreamedFrame::read_ne(reader)?)
    }
    let streamed_frames_copy = streamed_frames.clone();
    for (frame_index, streamed_frame) in streamed_frames.iter_mut().enumerate() {
        if frame_index == 0 || frame_index == 1 || frame_index == streamed_frames_copy.len() - 1 {
            continue;
        }
        for key in &mut streamed_frame.key_list {
            for i in (0..=frame_index - 1).rev() {
                let pre_frame = &streamed_frames_copy[i];
                if let Some(pre_framekey) = pre_frame.key_list.iter().find(|o| o.index == key.index)
                {
                    key.in_slope = pre_framekey
                        .calculate_next_in_slope(streamed_frame.time - pre_frame.time, &key);
                    break;
                }
            }
        }
    }
    Ok(streamed_frames)
}

pub fn streamed_clip_read_u32_buff(u32_buff: &Vec<u32>) -> anyhow::Result<Vec<StreamedFrame>> {
    let byte_buff_list: Vec<[u8; 4]> = u32_buff.iter().map(|u| u.to_ne_bytes()).collect();
    let streamed_clip_buff = byte_buff_list.concat();
    let mut streamed_clip_buff_reader = Cursor::new(streamed_clip_buff);
    streamed_clip_read_data(&mut streamed_clip_buff_reader)
}

pub fn animation_clip_binding_constant_find_binding(
    animation_clip_binding_constant: &TypeTreeObjectRef,
    index: usize,
) -> Option<TypeTreeObjectRef> {
    let mut curves = 0;
    for b in <Vec<TypeTreeObjectRef>>::try_cast_from(
        animation_clip_binding_constant,
        "/Base/genericBindings/Array",
    )
    .unwrap()
    {
        curves +=
            if i64::try_cast_from(&b, "/Base/typeID").unwrap() == ClassIDType::Transform as i64 {
                // 1 kBindTransformPosition
                // 2 kBindTransformRotation
                // 3 kBindTransformScale
                // 4 kBindTransformEuler
                match u64::try_cast_from(&b, "/Base/attribute").unwrap() {
                    1 | 3 | 4 => 3,
                    2 => 4,
                    _ => 1,
                }
            } else {
                1
            };

        if curves > index {
            return Some(b);
        }
    }
    None
}
