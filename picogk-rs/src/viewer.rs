//! Viewer support for interactive visualization

use crate::animation::{Animation, AnimationAction, AnimationQueue, AnimationType};
use crate::easing::EasingKind;
use crate::ffi;
use crate::log::LogFile;
use crate::types::{Matrix4x4, Vector2f, Vector3f};
use crate::utils::Utils;
use crate::{BBox3, ColorFloat, Error, Mesh, PolyLine, Result, Voxels};
use nalgebra::{Vector2, Vector3};
use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread;
use std::time::Instant;

#[derive(Clone)]
pub struct Viewer {
    inner: Arc<ViewerInner>,
}

struct ViewerInner {
    handle: *mut ffi::CViewer,
    log: LogFile,
    main_thread: thread::ThreadId,
    actions: Mutex<VecDeque<Box<dyn ViewerAction + Send>>>,
    animations: Mutex<AnimationQueue>,
    key_handlers: Mutex<VecDeque<Box<dyn KeyHandler + Send>>>,
    meshes: Mutex<Vec<MeshEntry>>,
    polylines: Mutex<Vec<PolyLineEntry>>,
    voxels: Mutex<HashMap<usize, VoxelsEntry>>,
    bbox: Mutex<BBox3>,
    idle: AtomicBool,
    view_state: Mutex<ViewState>,
    timelapse: Mutex<Option<TimeLapse>>,
}

#[derive(Clone, Copy)]
struct ViewState {
    elevation: f32,
    orbit: f32,
    fov: f32,
    zoom: f32,
    perspective: bool,
    background: ColorFloat,
    model_transform: Matrix4x4,
    model_view_projection: Matrix4x4,
    model_view_static: Matrix4x4,
    projection_static: Matrix4x4,
    mat_static: Matrix4x4,
    eye: Vector3<f32>,
    eye_static: Vector3<f32>,
    prev_mouse: Vector2<f32>,
    orbiting: bool,
}

struct MeshEntry {
    handle: usize,
    bbox: BBox3,
    triangles: usize,
    vertices: usize,
    _owned: Option<MeshKeepAlive>,
}

// Stored purely to keep native handles alive for the viewer lifetime; the value is never read.
#[allow(dead_code)]
enum MeshKeepAlive {
    Owned(Mesh),
    Shared(Arc<Mesh>),
}

struct PolyLineEntry {
    handle: usize,
    bbox: BBox3,
    _owned: Option<PolyLineKeepAlive>,
}

// Stored purely to keep native handles alive for the viewer lifetime; the value is never read.
#[allow(dead_code)]
enum PolyLineKeepAlive {
    Owned(PolyLine),
    Shared(Arc<PolyLine>),
}

struct VoxelsEntry {
    mesh_handle: usize,
    _keep_alive: Option<Arc<Voxels>>,
}

pub trait ViewerAction: Send {
    fn apply(&mut self, viewer: &Viewer) -> Result<()>;
}

/// Types that can be added to a [`Viewer`] via the C#-style `viewer.add(...)` method.
pub trait ViewerAdd {
    type Handle;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle;
}

/// Types that can be removed from a [`Viewer`] via the C#-style `viewer.remove(...)` method.
pub trait ViewerRemove {
    fn remove_from_viewer(self, viewer: &Viewer);
}

pub trait KeyHandler: Send {
    #[allow(clippy::too_many_arguments)]
    fn handle_event(
        &mut self,
        viewer: &Viewer,
        key: Key,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        cmd: bool,
    ) -> bool;
}

impl ViewerAdd for &Arc<Voxels> {
    type Handle = Arc<Voxels>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_voxels_shared(Arc::clone(self), group);
        Arc::clone(self)
    }
}

impl ViewerAdd for Voxels {
    type Handle = Arc<Voxels>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_voxels_owned(self, group)
    }
}

impl ViewerAdd for Arc<Voxels> {
    type Handle = Arc<Voxels>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_voxels_shared(Arc::clone(&self), group);
        self
    }
}

impl ViewerAdd for &Arc<Mesh> {
    type Handle = Arc<Mesh>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_mesh_shared(Arc::clone(self), group);
        Arc::clone(self)
    }
}

impl ViewerAdd for Mesh {
    type Handle = Arc<Mesh>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        let mesh = Arc::new(self);
        viewer.add_mesh_shared(Arc::clone(&mesh), group);
        mesh
    }
}

impl ViewerAdd for Arc<Mesh> {
    type Handle = Arc<Mesh>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_mesh_shared(Arc::clone(&self), group);
        self
    }
}

impl ViewerAdd for &Arc<PolyLine> {
    type Handle = Arc<PolyLine>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_polyline_shared(Arc::clone(self), group);
        Arc::clone(self)
    }
}

impl ViewerAdd for PolyLine {
    type Handle = Arc<PolyLine>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        let polyline = Arc::new(self);
        viewer.add_polyline_shared(Arc::clone(&polyline), group);
        polyline
    }
}

impl ViewerAdd for Arc<PolyLine> {
    type Handle = Arc<PolyLine>;
    fn add_to_viewer(self, viewer: &Viewer, group: i32) -> Self::Handle {
        viewer.add_polyline_shared(Arc::clone(&self), group);
        self
    }
}

impl ViewerRemove for &Arc<Voxels> {
    fn remove_from_viewer(self, viewer: &Viewer) {
        viewer.remove_voxels_shared(self);
    }
}

impl ViewerRemove for &Arc<Mesh> {
    fn remove_from_viewer(self, viewer: &Viewer) {
        viewer.remove_mesh_shared(self);
    }
}

impl ViewerRemove for &Arc<PolyLine> {
    fn remove_from_viewer(self, viewer: &Viewer) {
        viewer.remove_polyline_shared(self);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Key {
    Space = 32,
    Key0 = 48,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A = 65,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Esc = 256,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End = 269,
    F1 = 290,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl Key {
    fn from_code(code: i32) -> Option<Self> {
        match code {
            32 => Some(Key::Space),
            48 => Some(Key::Key0),
            49 => Some(Key::Key1),
            50 => Some(Key::Key2),
            51 => Some(Key::Key3),
            52 => Some(Key::Key4),
            53 => Some(Key::Key5),
            54 => Some(Key::Key6),
            55 => Some(Key::Key7),
            56 => Some(Key::Key8),
            57 => Some(Key::Key9),
            65 => Some(Key::A),
            66 => Some(Key::B),
            67 => Some(Key::C),
            68 => Some(Key::D),
            69 => Some(Key::E),
            70 => Some(Key::F),
            71 => Some(Key::G),
            72 => Some(Key::H),
            73 => Some(Key::I),
            74 => Some(Key::J),
            75 => Some(Key::K),
            76 => Some(Key::L),
            77 => Some(Key::M),
            78 => Some(Key::N),
            79 => Some(Key::O),
            80 => Some(Key::P),
            81 => Some(Key::Q),
            82 => Some(Key::R),
            83 => Some(Key::S),
            84 => Some(Key::T),
            85 => Some(Key::U),
            86 => Some(Key::V),
            87 => Some(Key::W),
            88 => Some(Key::X),
            89 => Some(Key::Y),
            90 => Some(Key::Z),
            256 => Some(Key::Esc),
            257 => Some(Key::Enter),
            258 => Some(Key::Tab),
            259 => Some(Key::Backspace),
            260 => Some(Key::Insert),
            261 => Some(Key::Delete),
            262 => Some(Key::Right),
            263 => Some(Key::Left),
            264 => Some(Key::Down),
            265 => Some(Key::Up),
            266 => Some(Key::PageUp),
            267 => Some(Key::PageDown),
            268 => Some(Key::Home),
            269 => Some(Key::End),
            290 => Some(Key::F1),
            291 => Some(Key::F2),
            292 => Some(Key::F3),
            293 => Some(Key::F4),
            294 => Some(Key::F5),
            295 => Some(Key::F6),
            296 => Some(Key::F7),
            297 => Some(Key::F8),
            298 => Some(Key::F9),
            299 => Some(Key::F10),
            300 => Some(Key::F11),
            301 => Some(Key::F12),
            _ => None,
        }
    }
}

pub struct KeyAction {
    action: Box<dyn ViewerAction + Send>,
    key: Key,
    pressed: bool,
    shift: bool,
    ctrl: bool,
    alt: bool,
    cmd: bool,
}

impl KeyAction {
    pub fn new(
        action: Box<dyn ViewerAction + Send>,
        key: Key,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        cmd: bool,
    ) -> Self {
        Self {
            action,
            key,
            pressed,
            shift,
            ctrl,
            alt,
            cmd,
        }
    }

    fn key_equals(
        &self,
        key: Key,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        cmd: bool,
    ) -> bool {
        self.key == key
            && self.pressed == pressed
            && self.shift == shift
            && self.ctrl == ctrl
            && self.alt == alt
            && self.cmd == cmd
    }

    fn apply(&mut self, viewer: &Viewer) {
        let _ = self.action.apply(viewer);
    }
}

pub struct KeyHandlerSet {
    actions: VecDeque<KeyAction>,
}

impl KeyHandlerSet {
    pub fn new() -> Self {
        Self {
            actions: VecDeque::new(),
        }
    }

    pub fn add_action(&mut self, action: KeyAction) {
        self.actions.push_front(action);
    }
}

impl Default for KeyHandlerSet {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyHandler for KeyHandlerSet {
    fn handle_event(
        &mut self,
        viewer: &Viewer,
        key: Key,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        cmd: bool,
    ) -> bool {
        for action in &mut self.actions {
            if action.key_equals(key, pressed, shift, ctrl, alt, cmd) {
                action.apply(viewer);
                return true;
            }
        }
        false
    }
}

pub enum RotateDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct RotateToNextRoundAngleAction {
    direction: RotateDirection,
}

impl RotateToNextRoundAngleAction {
    pub fn new(direction: RotateDirection) -> Self {
        Self { direction }
    }
}

impl ViewerAction for RotateToNextRoundAngleAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.remove_all_animations();

        let (orbit, elevation) = viewer.inner.view_angles();
        let mut target = Vector2::new(orbit, elevation);

        match self.direction {
            RotateDirection::Left | RotateDirection::Right => {
                let step = (target.x / 45.0) as i32;
                let delta = if matches!(self.direction, RotateDirection::Left) {
                    45.0
                } else {
                    -45.0
                };
                target.x = step as f32 * 45.0 + delta;
            }
            RotateDirection::Up | RotateDirection::Down => {
                let step = (target.y / 45.0) as i32;
                let delta = if matches!(self.direction, RotateDirection::Up) {
                    45.0
                } else {
                    -45.0
                };
                target.y = step as f32 * 45.0 + delta;
            }
        }

        let action = AnimViewRotate::new(viewer, Vector2::new(orbit, elevation), target);
        let anim = Animation::new(
            Box::new(action),
            0.7,
            AnimationType::Once,
            EasingKind::CubicOut,
        );
        viewer.add_animation(anim);
        Ok(())
    }
}

pub struct AnimGroupMatrixRotate {
    viewer: *const ViewerInner,
    group: i32,
    mat_init: Matrix4x4,
    axis: Vector3<f32>,
    degrees: f32,
}

impl AnimGroupMatrixRotate {
    pub fn new(
        viewer: &Viewer,
        group: i32,
        mat_init: Matrix4x4,
        axis: Vector3<f32>,
        degrees: f32,
    ) -> Self {
        Self {
            viewer: Arc::as_ptr(&viewer.inner),
            group,
            mat_init,
            axis,
            degrees,
        }
    }
}

impl AnimationAction for AnimGroupMatrixRotate {
    fn apply(&mut self, t: f32) {
        let angle = (t * self.degrees * std::f32::consts::PI) / 180.0;
        let axis = if self.axis.norm() > 1e-6 {
            self.axis.normalize()
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };
        let axis_unit = nalgebra::Unit::new_normalize(axis);
        let quat = nalgebra::UnitQuaternion::from_axis_angle(&axis_unit, angle);
        let mat_mul = Matrix4x4::from(quat.to_homogeneous());
        let mat = self.mat_init.multiply(&mat_mul);

        unsafe {
            if let Some(viewer) = self.viewer.as_ref() {
                viewer.set_group_matrix(self.group, mat);
            }
        }
    }
}

pub struct AnimViewRotate {
    viewer: *const ViewerInner,
    from: Vector2<f32>,
    to: Vector2<f32>,
}

impl AnimViewRotate {
    pub fn new(viewer: &Viewer, from: Vector2<f32>, to: Vector2<f32>) -> Self {
        Self {
            viewer: Arc::as_ptr(&viewer.inner),
            from,
            to,
        }
    }
}

impl AnimationAction for AnimViewRotate {
    fn apply(&mut self, t: f32) {
        let vec = self.from + (self.to - self.from) * t;
        unsafe {
            if let Some(viewer) = self.viewer.as_ref() {
                viewer.set_view_angles(vec.x, vec.y);
            }
        }
    }
}

struct SetGroupVisibleAction {
    group: i32,
    visible: bool,
}

impl ViewerAction for SetGroupVisibleAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.set_group_visible(self.group, self.visible);
        Ok(())
    }
}

struct SetGroupStaticAction {
    group: i32,
    is_static: bool,
}

impl ViewerAction for SetGroupStaticAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.set_group_static(self.group, self.is_static);
        Ok(())
    }
}

struct SetGroupMaterialAction {
    group: i32,
    color: ColorFloat,
    metallic: f32,
    roughness: f32,
}

impl ViewerAction for SetGroupMaterialAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer
            .inner
            .set_group_material(self.group, self.color, self.metallic, self.roughness);
        Ok(())
    }
}

struct SetGroupMatrixAction {
    group: i32,
    matrix: Matrix4x4,
}

impl ViewerAction for SetGroupMatrixAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.set_group_matrix(self.group, self.matrix);
        Ok(())
    }
}

struct RequestUpdateAction;

impl ViewerAction for RequestUpdateAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.request_update_now();
        Ok(())
    }
}

struct RequestScreenShotAction {
    path: String,
}

impl ViewerAction for RequestScreenShotAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.request_screenshot_now(&self.path);
        Ok(())
    }
}

struct RemoveVoxelsAction {
    voxels_handle: usize,
}

impl ViewerAction for RemoveVoxelsAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_remove_voxels(self.voxels_handle);
        Ok(())
    }
}

struct RemoveVoxelsSharedAction {
    voxels: Option<Arc<Voxels>>,
}

impl ViewerAction for RemoveVoxelsSharedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(voxels) = self.voxels.take() {
            viewer.inner.do_remove_voxels(voxels.handle() as usize);
        }
        Ok(())
    }
}

struct AddMeshOwnedAction {
    mesh: Option<Mesh>,
    group: i32,
}

impl ViewerAction for AddMeshOwnedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(mesh) = self.mesh.take() {
            viewer.inner.do_add_mesh_owned(mesh, self.group);
        }
        Ok(())
    }
}

struct AddMeshSharedAction {
    mesh: Option<Arc<Mesh>>,
    group: i32,
}

impl ViewerAction for AddMeshSharedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(mesh) = self.mesh.take() {
            viewer.inner.do_add_mesh_shared(mesh, self.group);
        }
        Ok(())
    }
}

struct AddMeshRefAction {
    handle: usize,
    bbox: BBox3,
    triangles: usize,
    vertices: usize,
    group: i32,
}

impl ViewerAction for AddMeshRefAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_add_mesh_handle(
            self.handle,
            self.bbox,
            self.triangles,
            self.vertices,
            self.group,
        );
        Ok(())
    }
}

struct RemoveMeshAction {
    handle: usize,
}

impl ViewerAction for RemoveMeshAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_remove_mesh_handle(self.handle);
        Ok(())
    }
}

struct RemoveMeshSharedAction {
    mesh: Option<Arc<Mesh>>,
}

impl ViewerAction for RemoveMeshSharedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(mesh) = self.mesh.take() {
            viewer.inner.do_remove_mesh_handle(mesh.handle() as usize);
        }
        Ok(())
    }
}

struct AddPolyLineOwnedAction {
    polyline: Option<PolyLine>,
    group: i32,
}

impl ViewerAction for AddPolyLineOwnedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(poly) = self.polyline.take() {
            viewer.inner.do_add_polyline_owned(poly, self.group);
        }
        Ok(())
    }
}

struct AddPolyLineSharedAction {
    polyline: Option<Arc<PolyLine>>,
    group: i32,
}

impl ViewerAction for AddPolyLineSharedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(poly) = self.polyline.take() {
            viewer.inner.do_add_polyline_shared(poly, self.group);
        }
        Ok(())
    }
}

struct AddPolyLineRefAction {
    handle: usize,
    bbox: BBox3,
    group: i32,
}

impl ViewerAction for AddPolyLineRefAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer
            .inner
            .do_add_polyline_handle(self.handle, self.bbox, self.group);
        Ok(())
    }
}

struct RemovePolyLineAction {
    handle: usize,
}

impl ViewerAction for RemovePolyLineAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_remove_polyline_handle(self.handle);
        Ok(())
    }
}

struct RemovePolyLineSharedAction {
    polyline: Option<Arc<PolyLine>>,
}

impl ViewerAction for RemovePolyLineSharedAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if let Some(poly) = self.polyline.take() {
            viewer
                .inner
                .do_remove_polyline_handle(poly.handle() as usize);
        }
        Ok(())
    }
}

struct RemoveAllObjectsAction;

impl ViewerAction for RemoveAllObjectsAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_remove_all_objects();
        Ok(())
    }
}

struct LogStatisticsAction;

impl ViewerAction for LogStatisticsAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        viewer.inner.do_log_statistics();
        Ok(())
    }
}

struct LoadLightSetupAction {
    diffuse: Vec<u8>,
    specular: Vec<u8>,
}

impl ViewerAction for LoadLightSetupAction {
    fn apply(&mut self, viewer: &Viewer) -> Result<()> {
        if !viewer
            .inner
            .load_light_setup_now(&self.diffuse, &self.specular)
        {
            let _ = viewer.inner.log.log("Failed to load light setup");
        }
        Ok(())
    }
}

struct TimeLapse {
    interval_ms: f32,
    path: PathBuf,
    file_name: String,
    current_frame: u32,
    paused: bool,
    start: Instant,
    next_time_ms: f32,
}

impl TimeLapse {
    fn new(
        interval_ms: f32,
        path: PathBuf,
        file_name: String,
        start_frame: u32,
        paused: bool,
    ) -> Self {
        let start = Instant::now();
        let next_time_ms = interval_ms;
        Self {
            interval_ms,
            path,
            file_name,
            current_frame: start_frame,
            paused,
            start,
            next_time_ms,
        }
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
        self.update_interval();
    }

    fn due(&mut self) -> Option<String> {
        if self.paused {
            return None;
        }

        let elapsed_ms = self.start.elapsed().as_millis() as f32;
        if elapsed_ms >= self.next_time_ms {
            let frame = format!("{:05}", self.current_frame);
            let filename = format!("{}{}.tga", self.file_name, frame);
            let path = self.path.join(filename);
            self.current_frame += 1;
            self.update_interval();
            return Some(path.to_string_lossy().to_string());
        }
        None
    }

    fn update_interval(&mut self) {
        let elapsed_ms = self.start.elapsed().as_millis() as f32;
        self.next_time_ms = elapsed_ms + self.interval_ms;
    }
}

static VIEWER_REGISTRY: OnceLock<Mutex<HashMap<usize, Weak<ViewerInner>>>> = OnceLock::new();
static INFO_LOG: OnceLock<Mutex<Option<LogFile>>> = OnceLock::new();

fn register_viewer(handle: *mut ffi::CViewer, viewer: &Arc<ViewerInner>) {
    let registry = VIEWER_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    registry
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(handle as usize, Arc::downgrade(viewer));
}

fn unregister_viewer(handle: *mut ffi::CViewer) {
    if let Some(registry) = VIEWER_REGISTRY.get() {
        registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&(handle as usize));
    }
}

fn with_viewer<F: FnOnce(&Viewer)>(handle: *mut ffi::CViewer, f: F) {
    if let Some(registry) = VIEWER_REGISTRY.get() {
        if let Some(weak) = registry
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&(handle as usize))
            .cloned()
        {
            if let Some(viewer) = weak.upgrade() {
                let wrapper = Viewer { inner: viewer };
                f(&wrapper);
            }
        }
    }
}

unsafe extern "C" fn info_cb(message: *const std::os::raw::c_char, _fatal: bool) {
    let msg = if message.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(message)
            .to_string_lossy()
            .to_string()
    };

    if let Some(log) = INFO_LOG.get() {
        if let Some(log) = log.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = log.log(&msg);
        }
    }
}

unsafe extern "C" fn update_cb(
    h_viewer: *mut ffi::CViewer,
    viewport: *const Vector2f,
    background: *mut ColorFloat,
    mat_mvp: *mut Matrix4x4,
    mat_model: *mut Matrix4x4,
    mat_static: *mut Matrix4x4,
    eye: *mut Vector3f,
    eye_static: *mut Vector3f,
) {
    let viewport = if viewport.is_null() {
        Vector2::new(1.0, 1.0)
    } else {
        Vector2::from(*viewport)
    };

    with_viewer(h_viewer, |viewer| {
        let _ = std::panic::catch_unwind(|| {
            viewer.inner.handle_update(
                viewport, background, mat_mvp, mat_model, mat_static, eye, eye_static,
            );
        });
    });
}

unsafe extern "C" fn key_pressed_cb(
    h_viewer: *mut ffi::CViewer,
    key: std::os::raw::c_int,
    _scancode: std::os::raw::c_int,
    action: std::os::raw::c_int,
    modifiers: std::os::raw::c_int,
) {
    with_viewer(h_viewer, |viewer| {
        viewer
            .inner
            .handle_key_pressed(viewer, key, action, modifiers);
    });
}

unsafe extern "C" fn mouse_moved_cb(h_viewer: *mut ffi::CViewer, pos: *const Vector2f) {
    if pos.is_null() {
        return;
    }
    let pos = Vector2::from(*pos);
    with_viewer(h_viewer, |viewer| {
        viewer.inner.handle_mouse_moved(pos);
    });
}

unsafe extern "C" fn mouse_button_cb(
    h_viewer: *mut ffi::CViewer,
    button: std::os::raw::c_int,
    action: std::os::raw::c_int,
    modifiers: std::os::raw::c_int,
    pos: *const Vector2f,
) {
    if pos.is_null() {
        return;
    }
    let pos = Vector2::from(*pos);
    with_viewer(h_viewer, |viewer| {
        viewer
            .inner
            .handle_mouse_button(button, action, modifiers, pos);
    });
}

unsafe extern "C" fn scroll_wheel_cb(
    h_viewer: *mut ffi::CViewer,
    wheel: *const Vector2f,
    pos: *const Vector2f,
) {
    if wheel.is_null() || pos.is_null() {
        return;
    }
    let wheel = Vector2::from(*wheel);
    let pos = Vector2::from(*pos);
    with_viewer(h_viewer, |viewer| {
        viewer.inner.handle_scroll_wheel(wheel, pos);
    });
}

unsafe extern "C" fn window_size_cb(h_viewer: *mut ffi::CViewer, size: *const Vector2f) {
    if size.is_null() {
        return;
    }
    let size = Vector2::from(*size);
    with_viewer(h_viewer, |viewer| {
        viewer.inner.handle_window_size(size);
    });
}

impl Viewer {
    pub fn new(title: &str, size: Vector2<f32>, log: LogFile) -> Result<Self> {
        let title = CString::new(title)
            .map_err(|_| Error::InvalidParameter("Window title contains null byte".to_string()))?;
        let size = Vector2f::from(size);

        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_hCreate(
                title.as_ptr(),
                &size as *const Vector2f,
                Some(info_cb),
                Some(update_cb),
                Some(key_pressed_cb),
                Some(mouse_moved_cb),
                Some(mouse_button_cb),
                Some(scroll_wheel_cb),
                Some(window_size_cb),
            )
        });

        if handle.is_null() {
            return Err(Error::NullPointer);
        }

        let inner = Arc::new(ViewerInner {
            handle,
            log: log.clone(),
            main_thread: thread::current().id(),
            actions: Mutex::new(VecDeque::new()),
            animations: Mutex::new(AnimationQueue::new()),
            key_handlers: Mutex::new(VecDeque::new()),
            meshes: Mutex::new(Vec::new()),
            polylines: Mutex::new(Vec::new()),
            voxels: Mutex::new(HashMap::new()),
            bbox: Mutex::new(BBox3::empty()),
            idle: AtomicBool::new(false),
            view_state: Mutex::new(ViewState {
                elevation: 30.0,
                orbit: 45.0,
                fov: 45.0,
                zoom: 1.0,
                perspective: true,
                background: ColorFloat::gray(0.3, 1.0),
                model_transform: Matrix4x4::identity(),
                model_view_projection: Matrix4x4::identity(),
                model_view_static: Matrix4x4::identity(),
                projection_static: Matrix4x4::identity(),
                mat_static: Matrix4x4::identity(),
                eye: Vector3::new(1.0, 1.0, 1.0),
                eye_static: Vector3::new(0.0, 10.0, 0.0),
                prev_mouse: Vector2::new(0.0, 0.0),
                orbiting: false,
            }),
            timelapse: Mutex::new(None),
        });

        register_viewer(handle, &inner);
        let info_log = INFO_LOG.get_or_init(|| Mutex::new(Some(log.clone())));
        *info_log.lock().unwrap_or_else(|e| e.into_inner()) = Some(log);

        let mut handler = KeyHandlerSet::new();
        handler.add_action(KeyAction::new(
            Box::new(RotateToNextRoundAngleAction::new(RotateDirection::Down)),
            Key::Down,
            false,
            false,
            false,
            false,
            false,
        ));
        handler.add_action(KeyAction::new(
            Box::new(RotateToNextRoundAngleAction::new(RotateDirection::Up)),
            Key::Up,
            false,
            false,
            false,
            false,
            false,
        ));
        handler.add_action(KeyAction::new(
            Box::new(RotateToNextRoundAngleAction::new(RotateDirection::Left)),
            Key::Left,
            false,
            false,
            false,
            false,
            false,
        ));
        handler.add_action(KeyAction::new(
            Box::new(RotateToNextRoundAngleAction::new(RotateDirection::Right)),
            Key::Right,
            false,
            false,
            false,
            false,
            false,
        ));

        inner.add_key_handler(Box::new(handler));

        Ok(Self { inner })
    }

    pub fn poll(&self) -> bool {
        self.inner.poll(self)
    }

    pub fn request_update(&self) {
        self.inner.enqueue_action(Box::new(RequestUpdateAction));
    }

    pub fn request_close(&self) {
        self.inner.request_close();
    }

    pub fn load_light_setup<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| Error::FileLoad(format!("Failed to read light setup: {}", e)))?;
        self.load_light_setup_from_reader(std::io::Cursor::new(data))
    }

    pub fn load_light_setup_from_reader<R: std::io::Read + std::io::Seek>(
        &self,
        reader: R,
    ) -> Result<()> {
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| Error::FileLoad(format!("Failed to read zip: {}", e)))?;
        let mut diffuse = Vec::new();
        let mut specular = Vec::new();

        {
            let mut file = archive
                .by_name("Diffuse.dds")
                .map_err(|_| Error::FileLoad("Diffuse.dds not found in zip".to_string()))?;
            std::io::Read::read_to_end(&mut file, &mut diffuse)
                .map_err(|e| Error::FileLoad(format!("Failed to read diffuse: {}", e)))?;
        }
        {
            let mut file = archive
                .by_name("Specular.dds")
                .map_err(|_| Error::FileLoad("Specular.dds not found in zip".to_string()))?;
            std::io::Read::read_to_end(&mut file, &mut specular)
                .map_err(|e| Error::FileLoad(format!("Failed to read specular: {}", e)))?;
        }

        self.inner
            .enqueue_action(Box::new(LoadLightSetupAction { diffuse, specular }));
        Ok(())
    }

    /// Add voxels to the viewer.
    ///
    /// Voxels are converted into a mesh **eagerly** at enqueue time, then the mesh is added to
    /// the native viewer on the next `poll()`. This avoids holding a borrowed voxels handle
    /// across the async action queue.
    pub fn add_voxels(&self, voxels: &Voxels, group: i32) {
        let voxels_handle = voxels.handle() as usize;
        let mesh_handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_hCreateFromVoxels(voxels_handle as *mut ffi::CVoxels)
        });
        if mesh_handle.is_null() {
            let _ = self.inner.log.log("Failed to create mesh from voxels");
            return;
        }

        let mesh = Mesh::from_handle(mesh_handle);
        let mesh_native = mesh.handle() as usize;

        self.inner
            .voxels
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                voxels_handle,
                VoxelsEntry {
                    mesh_handle: mesh_native,
                    _keep_alive: None,
                },
            );

        self.inner.enqueue_action(Box::new(AddMeshOwnedAction {
            mesh: Some(mesh),
            group,
        }));
    }

    /// Add voxels with shared ownership.
    ///
    /// The viewer keeps a clone of the `Arc` while the voxels are present in the viewer.
    pub fn add_voxels_shared(&self, voxels: Arc<Voxels>, group: i32) {
        let voxels_handle = voxels.handle() as usize;
        let mesh_handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_hCreateFromVoxels(voxels_handle as *mut ffi::CVoxels)
        });
        if mesh_handle.is_null() {
            let _ = self.inner.log.log("Failed to create mesh from voxels");
            return;
        }

        let mesh = Mesh::from_handle(mesh_handle);
        let mesh_native = mesh.handle() as usize;

        self.inner
            .voxels
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                voxels_handle,
                VoxelsEntry {
                    mesh_handle: mesh_native,
                    _keep_alive: Some(voxels),
                },
            );

        self.inner.enqueue_action(Box::new(AddMeshOwnedAction {
            mesh: Some(mesh),
            group,
        }));
    }

    /// Add voxels with owned lifetime (convenience wrapper around `Arc`).
    ///
    /// Returns an `Arc` that you can keep if you want to remove the voxels later.
    pub fn add_voxels_owned(&self, voxels: Voxels, group: i32) -> Arc<Voxels> {
        let voxels = Arc::new(voxels);
        self.add_voxels_shared(Arc::clone(&voxels), group);
        voxels
    }

    /// C#-style generic `Add(...)` wrapper.
    ///
    /// Returns an `Arc<...>` handle you can keep for later removal via `viewer.remove(&handle)`.
    ///
    /// This wrapper always uses an owned/shared path internally (so the added object is kept
    /// alive until it is removed from the viewer).
    pub fn add<T: ViewerAdd>(&self, object: T, group: i32) -> T::Handle {
        object.add_to_viewer(self, group)
    }

    pub fn remove_voxels(&self, voxels: &Voxels) {
        self.inner.enqueue_action(Box::new(RemoveVoxelsAction {
            voxels_handle: voxels.handle() as usize,
        }));
    }

    pub fn remove_voxels_shared(&self, voxels: &Arc<Voxels>) {
        self.inner
            .enqueue_action(Box::new(RemoveVoxelsSharedAction {
                voxels: Some(Arc::clone(voxels)),
            }));
    }

    /// Add a mesh by reference (handle-only).
    ///
    /// Prefer `add_mesh_owned` / `add_mesh_shared` unless you explicitly manage lifetimes.
    ///
    /// # Safety
    ///
    /// The viewer does not take ownership of `mesh`. The caller must ensure `mesh` remains alive
    /// until it is removed from the viewer **and** the remove action has been processed (i.e.
    /// after a subsequent `poll()`).
    pub unsafe fn add_mesh(&self, mesh: &Mesh, group: i32) {
        let bbox = mesh.bounding_box();
        let triangles = mesh.triangle_count();
        let vertices = mesh.vertex_count();
        self.inner.enqueue_action(Box::new(AddMeshRefAction {
            handle: mesh.handle() as usize,
            bbox,
            triangles,
            vertices,
            group,
        }));
    }

    pub fn add_mesh_owned(&self, mesh: Mesh, group: i32) {
        self.inner.enqueue_action(Box::new(AddMeshOwnedAction {
            mesh: Some(mesh),
            group,
        }));
    }

    /// Add a mesh with shared ownership.
    ///
    /// The viewer keeps a clone of the `Arc`, ensuring the mesh stays alive until removed.
    pub fn add_mesh_shared(&self, mesh: Arc<Mesh>, group: i32) {
        self.inner.enqueue_action(Box::new(AddMeshSharedAction {
            mesh: Some(mesh),
            group,
        }));
    }

    pub fn remove_mesh(&self, mesh: &Mesh) {
        self.inner.enqueue_action(Box::new(RemoveMeshAction {
            handle: mesh.handle() as usize,
        }));
    }

    pub fn remove_mesh_shared(&self, mesh: &Arc<Mesh>) {
        self.inner.enqueue_action(Box::new(RemoveMeshSharedAction {
            mesh: Some(Arc::clone(mesh)),
        }));
    }

    pub fn add_polyline(&self, polyline: PolyLine, group: i32) {
        self.inner.enqueue_action(Box::new(AddPolyLineOwnedAction {
            polyline: Some(polyline),
            group,
        }));
    }

    /// Add a polyline with shared ownership.
    ///
    /// The viewer keeps a clone of the `Arc`, ensuring the polyline stays alive until removed.
    pub fn add_polyline_shared(&self, polyline: Arc<PolyLine>, group: i32) {
        self.inner.enqueue_action(Box::new(AddPolyLineSharedAction {
            polyline: Some(polyline),
            group,
        }));
    }

    /// Add a polyline by reference (handle-only).
    ///
    /// Prefer `add_polyline` / `add_polyline_shared` unless you explicitly manage lifetimes.
    ///
    /// # Safety
    ///
    /// The viewer does not take ownership of `polyline`. The caller must ensure `polyline`
    /// remains alive until it is removed from the viewer **and** the remove action has been
    /// processed (i.e. after a subsequent `poll()`).
    pub unsafe fn add_polyline_ref(&self, polyline: &PolyLine, group: i32) {
        let bbox = polyline.bounding_box();
        self.inner.enqueue_action(Box::new(AddPolyLineRefAction {
            handle: polyline.handle() as usize,
            bbox,
            group,
        }));
    }

    pub fn remove_polyline(&self, polyline: &PolyLine) {
        self.inner.enqueue_action(Box::new(RemovePolyLineAction {
            handle: polyline.handle() as usize,
        }));
    }

    pub fn remove_polyline_shared(&self, polyline: &Arc<PolyLine>) {
        self.inner
            .enqueue_action(Box::new(RemovePolyLineSharedAction {
                polyline: Some(Arc::clone(polyline)),
            }));
    }

    /// C#-style generic `Remove(...)` wrapper.
    pub fn remove<T: ViewerRemove>(&self, object: T) {
        object.remove_from_viewer(self);
    }

    pub fn remove_all_objects(&self) {
        self.inner.enqueue_action(Box::new(RemoveAllObjectsAction));
    }

    pub fn request_screenshot(&self, path: &str) {
        self.inner.enqueue_action(Box::new(RequestScreenShotAction {
            path: path.to_string(),
        }));
    }

    /// C#-style alias for `request_screenshot`.
    pub fn request_screen_shot(&self, path: &str) {
        self.request_screenshot(path);
    }

    pub fn set_group_visible(&self, group: i32, visible: bool) {
        self.inner
            .enqueue_action(Box::new(SetGroupVisibleAction { group, visible }));
    }

    pub fn set_group_static(&self, group: i32, is_static: bool) {
        self.inner
            .enqueue_action(Box::new(SetGroupStaticAction { group, is_static }));
    }

    pub fn set_group_material(&self, group: i32, color: ColorFloat, metallic: f32, roughness: f32) {
        self.inner.enqueue_action(Box::new(SetGroupMaterialAction {
            group,
            color,
            metallic,
            roughness,
        }));
    }

    pub fn set_group_matrix(&self, group: i32, matrix: Matrix4x4) {
        self.inner
            .enqueue_action(Box::new(SetGroupMatrixAction { group, matrix }));
    }

    pub fn set_background_color(&self, color: ColorFloat) {
        self.inner.set_background_color(color);
    }

    pub fn adjust_view_angles(&self, orbit_delta: f32, elevation_delta: f32) {
        self.inner.adjust_view_angles(orbit_delta, elevation_delta);
    }

    pub fn set_view_angles(&self, orbit: f32, elevation: f32) {
        self.inner.set_view_angles(orbit, elevation);
    }

    pub fn set_fov(&self, fov: f32) {
        self.inner.set_fov(fov);
    }

    pub fn log_statistics(&self) {
        self.inner.enqueue_action(Box::new(LogStatisticsAction));
    }

    pub fn is_idle(&self) -> bool {
        self.inner.is_idle()
    }

    pub fn add_key_handler(&self, handler: Box<dyn KeyHandler + Send>) {
        self.inner.add_key_handler(handler);
    }

    pub fn add_animation(&self, anim: Animation) {
        self.inner.add_animation(anim);
    }

    pub fn remove_all_animations(&self) {
        self.inner.remove_all_animations();
    }

    pub fn start_timelapse(
        &self,
        interval_ms: f32,
        path: &str,
        file_name: &str,
        start_frame: u32,
        paused: bool,
    ) {
        self.inner
            .start_timelapse(interval_ms, path, file_name, start_frame, paused);
    }

    /// C#-style alias for `start_timelapse`.
    pub fn start_time_lapse(
        &self,
        interval_ms: f32,
        path: &str,
        file_name: Option<&str>,
        start_frame: Option<u32>,
        paused: Option<bool>,
    ) {
        self.start_timelapse(
            interval_ms,
            path,
            file_name.unwrap_or("frame_"),
            start_frame.unwrap_or(0),
            paused.unwrap_or(false),
        );
    }

    pub fn pause_timelapse(&self) {
        self.inner.pause_timelapse();
    }

    /// C#-style alias for `pause_timelapse`.
    pub fn pause_time_lapse(&self) {
        self.pause_timelapse();
    }

    pub fn resume_timelapse(&self) {
        self.inner.resume_timelapse();
    }

    /// C#-style alias for `resume_timelapse`.
    pub fn resume_time_lapse(&self) {
        self.resume_timelapse();
    }

    pub fn stop_timelapse(&self) {
        self.inner.stop_timelapse();
    }

    /// C#-style alias for `stop_timelapse`.
    pub fn stop_time_lapse(&self) {
        self.stop_timelapse();
    }
}

impl ViewerInner {
    fn enqueue_action(&self, action: Box<dyn ViewerAction + Send>) {
        self.actions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(action);
    }

    fn add_key_handler(&self, handler: Box<dyn KeyHandler + Send>) {
        self.key_handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_front(handler);
    }

    fn poll(&self, viewer: &Viewer) -> bool {
        if self.main_thread != thread::current().id() {
            let _ = self
                .log
                .log("Viewer::poll must be called from the main thread");
            return false;
        }

        thread::yield_now();

        let mut update_needed = false;
        if self
            .animations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pulse()
        {
            update_needed = true;
        }

        let actions = {
            let mut queue = self.actions.lock().unwrap_or_else(|e| e.into_inner());
            if queue.is_empty() {
                self.idle.store(true, Ordering::SeqCst);
            }
            queue.drain(..).collect::<Vec<_>>()
        };

        for mut action in actions {
            if let Err(err) = action.apply(viewer) {
                let _ = self.log.log(format!("Viewer action error: {}", err));
            }
            update_needed = true;
        }

        if let Some(path) = {
            let mut tl = self.timelapse.lock().unwrap_or_else(|e| e.into_inner());
            tl.as_mut().and_then(|tl| tl.due())
        } {
            self.request_screenshot_now(&path);
            update_needed = true;
        }

        if update_needed {
            self.request_update_now();
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Viewer_bPoll(self.handle) })
    }

    fn request_update_now(&self) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_RequestUpdate(self.handle);
        });
    }

    fn request_close(&self) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_RequestClose(self.handle);
        });
    }

    fn request_screenshot_now(&self, path: &str) {
        if let Ok(path) = CString::new(path) {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Viewer_RequestScreenShot(self.handle, path.as_ptr());
            });
        }
    }

    fn load_light_setup_now(&self, diffuse: &[u8], specular: &[u8]) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_bLoadLightSetup(
                self.handle,
                diffuse.as_ptr(),
                diffuse.len() as i32,
                specular.as_ptr(),
                specular.len() as i32,
            )
        })
    }

    fn set_group_visible(&self, group: i32, visible: bool) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_SetGroupVisible(self.handle, group, visible);
        });
    }

    fn set_group_static(&self, group: i32, is_static: bool) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_SetGroupStatic(self.handle, group, is_static);
        });
    }

    fn set_group_material(&self, group: i32, color: ColorFloat, metallic: f32, roughness: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_SetGroupMaterial(
                self.handle,
                group,
                &color as *const ColorFloat,
                metallic,
                roughness,
            );
        });
    }

    fn set_group_matrix(&self, group: i32, matrix: Matrix4x4) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_SetGroupMatrix(self.handle, group, &matrix as *const Matrix4x4);
        });
    }

    fn set_background_color(&self, color: ColorFloat) {
        self.view_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .background = color;
        self.request_update();
    }

    fn adjust_view_angles(&self, orbit_delta: f32, elevation_delta: f32) {
        let (orbit, elevation) = self.view_angles();
        self.set_view_angles(orbit + orbit_delta, elevation + elevation_delta);
    }

    fn set_view_angles(&self, orbit: f32, elevation: f32) {
        let mut state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());
        state.elevation = elevation;
        state.orbit = orbit;
        while state.orbit > 360.0 {
            state.orbit -= 360.0;
        }
        while state.orbit < 0.0 {
            state.orbit += 360.0;
        }
        drop(state);
        self.request_update();
    }

    fn set_fov(&self, fov: f32) {
        self.view_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .fov = fov;
        self.request_update();
    }

    fn add_animation(&self, anim: Animation) {
        self.animations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .add(anim);
    }

    fn remove_all_animations(&self) {
        self.animations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }

    fn view_angles(&self) -> (f32, f32) {
        let state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());
        (state.orbit, state.elevation)
    }

    fn request_update(&self) {
        self.enqueue_action(Box::new(RequestUpdateAction));
    }

    fn is_idle(&self) -> bool {
        let has_actions = !self
            .actions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty();
        if has_actions {
            self.idle.store(false, Ordering::SeqCst);
        }
        self.idle.load(Ordering::SeqCst)
    }

    fn do_add_mesh_owned(&self, mesh: Mesh, group: i32) {
        let handle = mesh.handle() as usize;
        let bbox = mesh.bounding_box();
        let triangles = mesh.triangle_count();
        let vertices = mesh.vertex_count();
        let entry = MeshEntry {
            handle,
            bbox,
            triangles,
            vertices,
            _owned: Some(MeshKeepAlive::Owned(mesh)),
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&bbox);
            self.meshes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddMesh(self.handle, group, handle as *mut ffi::CMesh);
        });
    }

    fn do_add_mesh_shared(&self, mesh: Arc<Mesh>, group: i32) {
        let handle = mesh.handle() as usize;
        let bbox = mesh.bounding_box();
        let triangles = mesh.triangle_count();
        let vertices = mesh.vertex_count();
        let entry = MeshEntry {
            handle,
            bbox,
            triangles,
            vertices,
            _owned: Some(MeshKeepAlive::Shared(mesh)),
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&entry.bbox);
            self.meshes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddMesh(self.handle, group, handle as *mut ffi::CMesh);
        });
    }

    fn do_add_mesh_handle(
        &self,
        handle: usize,
        bbox: BBox3,
        triangles: usize,
        vertices: usize,
        group: i32,
    ) {
        let entry = MeshEntry {
            handle,
            bbox,
            triangles,
            vertices,
            _owned: None,
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&entry.bbox);
            self.meshes
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddMesh(self.handle, group, handle as *mut ffi::CMesh);
        });
    }

    fn do_remove_mesh_handle(&self, handle: usize) {
        // IMPORTANT: keep the backing mesh alive while we call into the native viewer.
        // The mesh entry owns/keeps-alive the underlying handle for `*_owned/*_shared` add paths.
        let removed = {
            let mut meshes = self.meshes.lock().unwrap_or_else(|e| e.into_inner());
            meshes
                .iter()
                .position(|m| m.handle == handle)
                .map(|index| meshes.remove(index))
        };
        if removed.is_none() {
            let _ = self.log.log("Tried to remove mesh that was never added");
            return;
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_RemoveMesh(self.handle, handle as *mut ffi::CMesh);
        });
        self.recalculate_bbox();
        self.request_update();
    }

    fn do_add_polyline_owned(&self, polyline: PolyLine, group: i32) {
        let handle = polyline.handle() as usize;
        let bbox = polyline.bounding_box();
        let entry = PolyLineEntry {
            handle,
            bbox,
            _owned: Some(PolyLineKeepAlive::Owned(polyline)),
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&bbox);
            self.polylines
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddPolyLine(self.handle, group, handle as *mut ffi::CPolyLine);
        });
    }

    fn do_add_polyline_shared(&self, polyline: Arc<PolyLine>, group: i32) {
        let handle = polyline.handle() as usize;
        let bbox = polyline.bounding_box();
        let entry = PolyLineEntry {
            handle,
            bbox,
            _owned: Some(PolyLineKeepAlive::Shared(polyline)),
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&bbox);
            self.polylines
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddPolyLine(self.handle, group, handle as *mut ffi::CPolyLine);
        });
    }

    fn do_add_polyline_handle(&self, handle: usize, bbox: BBox3, group: i32) {
        let entry = PolyLineEntry {
            handle,
            bbox,
            _owned: None,
        };

        {
            self.bbox
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .include_bbox(&entry.bbox);
            self.polylines
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(entry);
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_AddPolyLine(self.handle, group, handle as *mut ffi::CPolyLine);
        });
    }

    fn do_remove_polyline_handle(&self, handle: usize) {
        // IMPORTANT: keep the backing polyline alive while we call into the native viewer.
        let removed = {
            let mut polylines = self.polylines.lock().unwrap_or_else(|e| e.into_inner());
            polylines
                .iter()
                .position(|p| p.handle == handle)
                .map(|index| polylines.remove(index))
        };
        if removed.is_none() {
            let _ = self
                .log
                .log("Tried to remove polyline that was never added");
            return;
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Viewer_RemovePolyLine(self.handle, handle as *mut ffi::CPolyLine);
        });
        self.recalculate_bbox();
        self.request_update();
    }

    fn do_remove_voxels(&self, voxels_handle: usize) {
        let entry = self
            .voxels
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&voxels_handle);
        if let Some(entry) = entry {
            self.do_remove_mesh_handle(entry.mesh_handle);
        } else {
            let _ = self.log.log("Tried to remove voxels that were never added");
        }
    }

    fn do_remove_all_objects(&self) {
        {
            let mut polylines = self.polylines.lock().unwrap_or_else(|e| e.into_inner());
            for poly in polylines.iter() {
                crate::ffi_lock::with_ffi_lock(|| unsafe {
                    ffi::Viewer_RemovePolyLine(self.handle, poly.handle as *mut ffi::CPolyLine);
                });
            }
            polylines.clear();
        }

        {
            self.voxels
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
        }

        {
            let mut meshes = self.meshes.lock().unwrap_or_else(|e| e.into_inner());
            for mesh in meshes.iter() {
                crate::ffi_lock::with_ffi_lock(|| unsafe {
                    ffi::Viewer_RemoveMesh(self.handle, mesh.handle as *mut ffi::CMesh);
                });
            }
            meshes.clear();
        }

        self.recalculate_bbox();
    }

    fn do_log_statistics(&self) {
        let mut triangles = 0.0f32;
        let mut vertices = 0.0f32;
        let mut mesh_count = 0u64;

        {
            let meshes = self.meshes.lock().unwrap_or_else(|e| e.into_inner());
            for mesh in meshes.iter() {
                triangles += mesh.triangles as f32;
                vertices += mesh.vertices as f32;
                mesh_count += 1;
            }
        }

        let mut unit = "";
        if triangles > 1000.0 {
            unit = "K";
            triangles /= 1000.0;
            vertices /= 1000.0;

            if triangles > 1000.0 {
                unit = "mio";
                triangles /= 1000.0;
                vertices /= 1000.0;
            }
        }

        let _ = self.log.log("Viewer Stats:");
        let _ = self.log.log(format!("   Number of Meshes: {}", mesh_count));
        let _ = self.log.log(format!(
            "   Voxel Objects:    {}",
            self.voxels.lock().unwrap_or_else(|e| e.into_inner()).len()
        ));
        let _ = self
            .log
            .log(format!("   Total Triangles:  {:.1}{}", triangles, unit));
        let _ = self
            .log
            .log(format!("   Total Vertices:   {:.1}{}", vertices, unit));
        let _ = self.log.log(format!(
            "   Bounding Box:     {}",
            self.bbox.lock().unwrap_or_else(|e| e.into_inner())
        ));
    }

    fn recalculate_bbox(&self) {
        let mut bbox = BBox3::empty();
        {
            let meshes = self.meshes.lock().unwrap_or_else(|e| e.into_inner());
            for mesh in meshes.iter() {
                bbox.include_bbox(&mesh.bbox);
            }
        }
        {
            let polylines = self.polylines.lock().unwrap_or_else(|e| e.into_inner());
            for poly in polylines.iter() {
                bbox.include_bbox(&poly.bbox);
            }
        }
        *self.bbox.lock().unwrap_or_else(|e| e.into_inner()) = bbox;
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_update(
        &self,
        viewport: Vector2<f32>,
        background: *mut ColorFloat,
        mat_mvp: *mut Matrix4x4,
        mat_model: *mut Matrix4x4,
        mat_static: *mut Matrix4x4,
        eye: *mut Vector3f,
        eye_static: *mut Vector3f,
    ) {
        let bbox = *self.bbox.lock().unwrap_or_else(|e| e.into_inner());
        let mut state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());

        if !bbox.is_empty() {
            let center = bbox.center();
            let f_r = (bbox.max() - center).norm() * 3.0 * state.zoom;
            let f_r_elev = (state.elevation * std::f32::consts::PI / 180.0).cos() * f_r;

            state.eye.x = (state.orbit * std::f32::consts::PI / 180.0).cos() * f_r_elev;
            state.eye.y = (state.orbit * std::f32::consts::PI / 180.0).sin() * f_r_elev;
            state.eye.z = (state.elevation * std::f32::consts::PI / 180.0).sin() * f_r;

            let f_far = (center - state.eye).norm() * 2.0;
            let mat_view = Utils::mat_look_at(state.eye, center);
            let mat_proj = if state.perspective {
                perspective_fov(state.fov, viewport.x / viewport.y, 0.1, f_far)
            } else {
                orthographic(bbox.size().x * 2.0, bbox.size().y * 2.0, 0.1, f_far)
            };

            state.model_view_static =
                Utils::mat_look_at(state.eye_static, Vector3::new(0.0, 0.0, 0.0));
            state.projection_static =
                orthographic(100.0 * viewport.x / viewport.y, 100.0, 0.1, 100.0);

            state.model_view_projection = mat_view.multiply(&mat_proj);
            state.mat_static = state.model_view_static.multiply(&state.projection_static);
        }

        unsafe {
            if !background.is_null() {
                *background = state.background;
            }
            if !mat_mvp.is_null() {
                *mat_mvp = state.model_view_projection;
            }
            if !mat_model.is_null() {
                *mat_model = state.model_transform;
            }
            if !mat_static.is_null() {
                *mat_static = state.mat_static;
            }
            if !eye.is_null() {
                *eye = Vector3f::from(state.eye);
            }
            if !eye_static.is_null() {
                *eye_static = Vector3f::from(state.eye_static);
            }
        }
    }

    fn handle_key_pressed(&self, viewer: &Viewer, key: i32, action: i32, modifiers: i32) {
        if action != 0 && action != 1 {
            return;
        }
        let key = match Key::from_code(key) {
            Some(key) => key,
            None => return,
        };

        let pressed = action == 1;
        let shift = (modifiers & 0x0001) != 0;
        let ctrl = (modifiers & 0x0002) != 0;
        let alt = (modifiers & 0x0004) != 0;
        let cmd = (modifiers & 0x0008) != 0;

        let mut handlers = self.key_handlers.lock().unwrap_or_else(|e| e.into_inner());
        for handler in handlers.iter_mut() {
            if handler.handle_event(viewer, key, pressed, shift, ctrl, alt, cmd) {
                return;
            }
        }
    }

    fn handle_mouse_moved(&self, pos: Vector2<f32>) {
        let mut state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());
        if state.orbiting {
            let dist = pos - state.prev_mouse;
            state.prev_mouse = pos;
            drop(state);
            self.adjust_view_angles(-dist.x / 2.0, dist.y / 2.0);
            self.remove_all_animations();
        }
    }

    fn handle_mouse_button(&self, _button: i32, action: i32, _modifiers: i32, pos: Vector2<f32>) {
        let mut state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());
        if action == 1 {
            state.orbiting = true;
            state.prev_mouse = pos;
            drop(state);
            self.remove_all_animations();
        } else if action == 0 {
            state.orbiting = false;
        }
    }

    fn handle_scroll_wheel(&self, wheel: Vector2<f32>, _pos: Vector2<f32>) {
        let mut state = self.view_state.lock().unwrap_or_else(|e| e.into_inner());
        state.zoom -= wheel.y / 50.0;
        if state.zoom < 0.1 {
            state.zoom = 0.1;
        }
        drop(state);
        self.request_update();
    }

    fn handle_window_size(&self, _size: Vector2<f32>) {
        self.request_update();
    }

    fn start_timelapse(
        &self,
        interval_ms: f32,
        path: &str,
        file_name: &str,
        start_frame: u32,
        paused: bool,
    ) {
        let mut tl = self.timelapse.lock().unwrap_or_else(|e| e.into_inner());
        *tl = Some(TimeLapse::new(
            interval_ms,
            PathBuf::from(path),
            file_name.to_string(),
            start_frame,
            paused,
        ));
    }

    fn pause_timelapse(&self) {
        if let Some(tl) = self
            .timelapse
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
        {
            tl.pause();
        }
    }

    fn resume_timelapse(&self) {
        if let Some(tl) = self
            .timelapse
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
        {
            tl.resume();
        }
    }

    fn stop_timelapse(&self) {
        *self.timelapse.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

impl Drop for ViewerInner {
    fn drop(&mut self) {
        unregister_viewer(self.handle);
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Viewer_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for ViewerInner {}
unsafe impl Sync for ViewerInner {}

fn perspective_fov(fov_deg: f32, aspect: f32, near: f32, far: f32) -> Matrix4x4 {
    let fov = fov_deg * std::f32::consts::PI / 180.0;
    let y_scale = 1.0 / (fov / 2.0).tan();
    let x_scale = y_scale / aspect;
    let z_range = far - near;
    let m33 = -far / z_range;
    let m43 = -(far * near) / z_range;

    Matrix4x4 {
        m11: x_scale,
        m12: 0.0,
        m13: 0.0,
        m14: 0.0,
        m21: 0.0,
        m22: y_scale,
        m23: 0.0,
        m24: 0.0,
        m31: 0.0,
        m32: 0.0,
        m33,
        m34: -1.0,
        m41: 0.0,
        m42: 0.0,
        m43,
        m44: 0.0,
    }
}

fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Matrix4x4 {
    let m33 = 1.0 / (near - far);
    let m43 = near / (near - far);

    Matrix4x4 {
        m11: 2.0 / width,
        m12: 0.0,
        m13: 0.0,
        m14: 0.0,
        m21: 0.0,
        m22: 2.0 / height,
        m23: 0.0,
        m24: 0.0,
        m31: 0.0,
        m32: 0.0,
        m33,
        m34: 0.0,
        m41: 0.0,
        m42: 0.0,
        m43,
        m44: 1.0,
    }
}
