// src/lib.rs
use std::collections::HashMap;

struct ReferenceFrameBuffer {
    max_size: usize,
    frames: HashMap<u32, Vec<u8>>,
    short_term_ids: Vec<u32>,
    long_term_ids: Vec<u32>,
    next_id: u32,
}

impl ReferenceFrameBuffer {
    fn new(max_size: usize) -> Self {
        ReferenceFrameBuffer {
            max_size,
            frames: HashMap::new(),
            short_term_ids: Vec::new(),
            long_term_ids: Vec::new(),
            next_id: 0,
        }
    }

    fn add_frame(&mut self, frame_data: Vec<u8>, is_long_term: bool) -> u32 {
        let frame_id = self.next_id;
        self.next_id += 1;

        self.frames.insert(frame_id, frame_data);

        if is_long_term {
            self.long_term_ids.push(frame_id);
        } else {
            self.short_term_ids.push(frame_id);
        }

        self.enforce_size_limit();
        frame_id
    }

    fn promote_to_long_term(&mut self, frame_id: u32) -> bool {
        if !self.frames.contains_key(&frame_id) || self.long_term_ids.contains(&frame_id) {
            return false;
        }

        if let Some(pos) = self.short_term_ids.iter().position(|&id| id == frame_id) {
            self.short_term_ids.remove(pos);
            self.long_term_ids.push(frame_id);
            return true;
        }

        false
    }

    fn get_frame(&self, frame_id: u32) -> Option<&Vec<u8>> {
        self.frames.get(&frame_id)
    }

    fn enforce_size_limit(&mut self) {
        while self.frames.len() > self.max_size {
            if !self.short_term_ids.is_empty() {
                let old_id = self.short_term_ids.remove(0);
                self.frames.remove(&old_id);
            } else if !self.long_term_ids.is_empty() {
                let old_id = self.long_term_ids.remove(0);
                self.frames.remove(&old_id);
            }
        }
    }
}

/*
C-API
 */

// src/lib.rs (Fortsetzung)
use libc::{c_int, c_uint, c_void, size_t};
use std::ptr;

// Opaque Typ für den Buffer
pub struct BufferHandle {
    buffer: ReferenceFrameBuffer,
}

#[no_mangle]
pub extern "C" fn delta_create_buffer(max_size: size_t) -> *mut c_void {
    let buffer = ReferenceFrameBuffer::new(max_size);
    let handle = Box::new(BufferHandle { buffer });
    Box::into_raw(handle) as *mut c_void
}

#[no_mangle]
pub extern "C" fn delta_destroy_buffer(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut BufferHandle);
        }
    }
}

#[no_mangle]
pub extern "C" fn delta_add_frame(
    handle: *mut c_void,
    frame_data: *const u8,
    frame_size: size_t,
    is_long_term: c_int,
) -> c_uint {
    if handle.is_null() || frame_data.is_null() {
        return 0;
    }

    unsafe {
        let buffer_handle = &mut *(handle as *mut BufferHandle);
        let data = std::slice::from_raw_parts(frame_data, frame_size).to_vec();
        buffer_handle.buffer.add_frame(data, is_long_term != 0)
    }
}

#[no_mangle]
pub extern "C" fn delta_promote_to_long_term(handle: *mut c_void, frame_id: c_uint) -> c_int {
    if handle.is_null() {
        return 0;
    }

    unsafe {
        let buffer_handle = &mut *(handle as *mut BufferHandle);
        if buffer_handle.buffer.promote_to_long_term(frame_id) {
            1
        } else {
            0
        }
    }
}

// Struktur für den Frame-Zugriff aus C
#[repr(C)]
pub struct FrameData {
    data: *const u8,
    size: size_t,
}

#[no_mangle]
pub extern "C" fn delta_get_frame(handle: *mut c_void, frame_id: c_uint) -> FrameData {
    if handle.is_null() {
        return FrameData {
            data: ptr::null(),
            size: 0,
        };
    }

    unsafe {
        let buffer_handle = &*(handle as *mut BufferHandle);
        if let Some(frame) = buffer_handle.buffer.get_frame(frame_id) {
            FrameData {
                data: frame.as_ptr(),
                size: frame.len(),
            }
        } else {
            FrameData {
                data: ptr::null(),
                size: 0,
            }
        }
    }
}

// inter-frame-prediction

// src/lib.rs (Fortsetzung)

// Eine vereinfachte Inter-Prediction-Implementierung für die C-API
struct InterPrediction {
    // Hier würden wir die tatsächliche Implementierungslogik hinzufügen
}

impl InterPrediction {
    fn new() -> Self {
        InterPrediction {}
    }

    fn predict_frame(&self, current_frame: &[u8], reference_frames: &[&[u8]]) -> Vec<u8> {
        // Vereinfachte Implementierung für das Beispiel
        // Hier würde die eigentliche Bewegungsschätzung und -kompensation erfolgen
        let mut predicted = current_frame.to_vec();
        // Dummy-Implementierung, die einfach das aktuelle Frame zurückgibt
        predicted
    }
}

// Opaque Typ für den Predictor
pub struct PredictorHandle {
    predictor: InterPrediction,
}

#[no_mangle]
pub extern "C" fn delta_create_predictor() -> *mut c_void {
    let predictor = InterPrediction::new();
    let handle = Box::new(PredictorHandle { predictor });
    Box::into_raw(handle) as *mut c_void
}

#[no_mangle]
pub extern "C" fn delta_destroy_predictor(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut PredictorHandle);
        }
    }
}

#[no_mangle]
pub extern "C" fn delta_predict_frame(
    predictor_handle: *mut c_void,
    current_frame: *const u8,
    current_size: size_t,
    ref_frames: *const *const u8,
    ref_sizes: *const size_t,
    num_refs: size_t,
    output_buffer: *mut u8,
    output_size: size_t,
) -> size_t {
    if predictor_handle.is_null() || current_frame.is_null() || output_buffer.is_null() {
        return 0;
    }

    unsafe {
        let predictor = &(*(predictor_handle as *mut PredictorHandle)).predictor;
        let current = std::slice::from_raw_parts(current_frame, current_size);

        // Sammle Referenzframes
        let mut references = Vec::with_capacity(num_refs);
        for i in 0..num_refs {
            let ref_frame = *ref_frames.add(i);
            let ref_size = *ref_sizes.add(i);
            if !ref_frame.is_null() {
                let frame_slice = std::slice::from_raw_parts(ref_frame, ref_size);
                references.push(frame_slice);
            }
        }

        let reference_slices: Vec<&[u8]> = references.iter().map(|s| &s[..]).collect();
        let predicted = predictor.predict_frame(current, &reference_slices);

        let copy_size = std::cmp::min(predicted.len(), output_size);
        ptr::copy_nonoverlapping(predicted.as_ptr(), output_buffer, copy_size);

        copy_size
    }
}
