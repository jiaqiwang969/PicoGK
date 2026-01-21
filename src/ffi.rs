//! FFI bindings to PicoGK C++ library
//!
//! This module provides low-level bindings to the PicoGK C++ runtime.
//! These functions are unsafe and should not be called directly by users.

use std::os::raw::{c_char, c_float, c_int, c_void};

// Opaque types representing C++ objects
pub type CVoxels = c_void;
pub type CMesh = c_void;
pub type CLattice = c_void;
pub type CScalarField = c_void;
pub type CVectorField = c_void;
pub type CVdbFile = c_void;
pub type CMetadata = c_void;
pub type CPolyLine = c_void;
pub type CViewer = c_void;

pub type ImplicitCallback = Option<unsafe extern "C" fn(*const crate::types::Vector3f) -> c_float>;
pub type ScalarFieldTraverseCallback =
    Option<unsafe extern "C" fn(*const crate::types::Vector3f, c_float)>;
pub type VectorFieldTraverseCallback =
    Option<unsafe extern "C" fn(*const crate::types::Vector3f, *const crate::types::Vector3f)>;

pub type ViewerInfoCallback = Option<unsafe extern "C" fn(*const c_char, bool)>;
pub type ViewerUpdateCallback = Option<
    unsafe extern "C" fn(
        *mut CViewer,
        *const crate::types::Vector2f,
        *mut crate::types::ColorFloat,
        *mut crate::types::Matrix4x4,
        *mut crate::types::Matrix4x4,
        *mut crate::types::Matrix4x4,
        *mut crate::types::Vector3f,
        *mut crate::types::Vector3f,
    ),
>;
pub type ViewerKeyPressedCallback =
    Option<unsafe extern "C" fn(*mut CViewer, c_int, c_int, c_int, c_int)>;
pub type ViewerMouseMovedCallback =
    Option<unsafe extern "C" fn(*mut CViewer, *const crate::types::Vector2f)>;
pub type ViewerMouseButtonCallback =
    Option<unsafe extern "C" fn(*mut CViewer, c_int, c_int, c_int, *const crate::types::Vector2f)>;
pub type ViewerScrollWheelCallback = Option<
    unsafe extern "C" fn(
        *mut CViewer,
        *const crate::types::Vector2f,
        *const crate::types::Vector2f,
    ),
>;
pub type ViewerWindowSizeCallback =
    Option<unsafe extern "C" fn(*mut CViewer, *const crate::types::Vector2f)>;

extern "C" {
    // ========================================
    // Library functions
    // ========================================

    pub fn Library_Init(voxel_size_mm: c_float);
    pub fn Library_Destroy();
    pub fn Library_GetName(buffer: *mut c_char);
    pub fn Library_GetVersion(buffer: *mut c_char);
    pub fn Library_GetBuildInfo(buffer: *mut c_char);
    pub fn Library_VoxelsToMm(
        voxel_coordinate: *const crate::types::Vector3f,
        mm_coordinate: *mut crate::types::Vector3f,
    );
    pub fn Library_MmToVoxels(
        mm_coordinate: *const crate::types::Vector3f,
        voxel_coordinate: *mut crate::types::Vector3f,
    );

    // ========================================
    // Voxels functions
    // ========================================

    pub fn Voxels_hCreate() -> *mut CVoxels;
    pub fn Voxels_hCreateCopy(source: *const CVoxels) -> *mut CVoxels;
    pub fn Voxels_Destroy(handle: *mut CVoxels);
    pub fn Voxels_bIsValid(handle: *const CVoxels) -> bool;

    // Boolean operations
    pub fn Voxels_BoolAdd(this: *mut CVoxels, operand: *const CVoxels);
    pub fn Voxels_BoolSubtract(this: *mut CVoxels, operand: *const CVoxels);
    pub fn Voxels_BoolIntersect(this: *mut CVoxels, operand: *const CVoxels);
    pub fn Voxels_BoolAddSmooth(
        this: *mut CVoxels,
        operand: *const CVoxels,
        smooth_distance: c_float,
    );

    // Morphological operations
    pub fn Voxels_Offset(this: *mut CVoxels, dist_mm: c_float);
    pub fn Voxels_DoubleOffset(this: *mut CVoxels, dist1_mm: c_float, dist2_mm: c_float);
    pub fn Voxels_TripleOffset(this: *mut CVoxels, dist_mm: c_float);
    pub fn Voxels_Gaussian(this: *mut CVoxels, dist_mm: c_float);
    pub fn Voxels_Median(this: *mut CVoxels, size_mm: c_float);
    pub fn Voxels_Mean(this: *mut CVoxels, size_mm: c_float);

    // Rendering
    pub fn Voxels_RenderMesh(this: *mut CVoxels, mesh: *const CMesh);
    pub fn Voxels_RenderImplicit(
        this: *mut CVoxels,
        bounds: *const crate::BBox3,
        callback: ImplicitCallback,
    );
    pub fn Voxels_IntersectImplicit(this: *mut CVoxels, callback: ImplicitCallback);
    pub fn Voxels_RenderLattice(this: *mut CVoxels, lattice: *const CLattice);
    pub fn Voxels_ProjectZSlice(this: *mut CVoxels, start_z_mm: c_float, end_z_mm: c_float);

    // Query operations
    pub fn Voxels_CalculateProperties(
        this: *const CVoxels,
        volume_mm3: *mut c_float,
        bbox: *mut crate::BBox3,
    );
    pub fn Voxels_GetSurfaceNormal(
        this: *const CVoxels,
        surface_point: *const crate::types::Vector3f,
        surface_normal: *mut crate::types::Vector3f,
    );
    pub fn Voxels_bClosestPointOnSurface(
        this: *const CVoxels,
        search_point: *const crate::types::Vector3f,
        surface_point: *mut crate::types::Vector3f,
    ) -> bool;
    pub fn Voxels_bRayCastToSurface(
        this: *const CVoxels,
        ray_origin: *const crate::types::Vector3f,
        ray_direction: *const crate::types::Vector3f,
        surface_point: *mut crate::types::Vector3f,
    ) -> bool;
    pub fn Voxels_bIsInside(
        this: *const CVoxels,
        test_point: *const crate::types::Vector3f,
    ) -> bool;
    pub fn Voxels_bIsEqual(this: *const CVoxels, other: *const CVoxels) -> bool;
    pub fn Voxels_GetVoxelDimensions(
        this: *const CVoxels,
        x_origin: *mut c_int,
        y_origin: *mut c_int,
        z_origin: *mut c_int,
        x_size: *mut c_int,
        y_size: *mut c_int,
        z_size: *mut c_int,
    );
    pub fn Voxels_GetSlice(
        this: *const CVoxels,
        z_slice: c_int,
        buffer: *mut c_float,
        background_value: *mut c_float,
    );
    pub fn Voxels_GetInterpolatedSlice(
        this: *const CVoxels,
        z_slice: c_float,
        buffer: *mut c_float,
        background_value: *mut c_float,
    );

    // ========================================
    // Mesh functions
    // ========================================

    pub fn Mesh_hCreate() -> *mut CMesh;
    pub fn Mesh_hCreateFromVoxels(voxels: *const CVoxels) -> *mut CMesh;
    pub fn Mesh_Destroy(handle: *mut CMesh);
    pub fn Mesh_bIsValid(handle: *const CMesh) -> bool;

    pub fn Mesh_nAddVertex(this: *mut CMesh, vertex: *const crate::types::Vector3f) -> c_int;

    pub fn Mesh_nAddTriangle(this: *mut CMesh, tri: *const crate::Triangle) -> c_int;

    pub fn Mesh_nVertexCount(this: *const CMesh) -> c_int;
    pub fn Mesh_nTriangleCount(this: *const CMesh) -> c_int;

    pub fn Mesh_GetVertex(this: *const CMesh, index: c_int, vertex: *mut crate::types::Vector3f);

    pub fn Mesh_GetTriangle(this: *const CMesh, index: c_int, tri: *mut crate::Triangle);

    pub fn Mesh_GetTriangleV(
        this: *const CMesh,
        index: c_int,
        vec_a: *mut crate::types::Vector3f,
        vec_b: *mut crate::types::Vector3f,
        vec_c: *mut crate::types::Vector3f,
    );

    pub fn Mesh_GetBoundingBox(this: *const CMesh, bbox: *mut crate::BBox3);

    // ========================================
    // PolyLine functions
    // ========================================

    pub fn PolyLine_hCreate(color: *const crate::types::ColorFloat) -> *mut CPolyLine;
    pub fn PolyLine_Destroy(handle: *mut CPolyLine);
    pub fn PolyLine_bIsValid(handle: *const CPolyLine) -> bool;
    pub fn PolyLine_nAddVertex(
        this: *mut CPolyLine,
        vertex: *const crate::types::Vector3f,
    ) -> c_int;
    pub fn PolyLine_nVertexCount(this: *const CPolyLine) -> c_int;
    pub fn PolyLine_GetVertex(
        this: *const CPolyLine,
        index: c_int,
        vertex: *mut crate::types::Vector3f,
    );
    pub fn PolyLine_GetColor(this: *const CPolyLine, color: *mut crate::types::ColorFloat);

    // ========================================
    // Lattice functions
    // ========================================

    pub fn Lattice_hCreate() -> *mut CLattice;
    pub fn Lattice_Destroy(handle: *mut CLattice);
    pub fn Lattice_bIsValid(handle: *const CLattice) -> bool;

    pub fn Lattice_AddSphere(
        this: *mut CLattice,
        center: *const crate::types::Vector3f,
        radius: c_float,
    );

    pub fn Lattice_AddBeam(
        this: *mut CLattice,
        vec_a: *const crate::types::Vector3f,
        vec_b: *const crate::types::Vector3f,
        r1: c_float,
        r2: c_float,
        round_cap: bool,
    );

    // ========================================
    // ScalarField functions
    // ========================================

    pub fn ScalarField_hCreate() -> *mut CScalarField;
    pub fn ScalarField_hCreateCopy(source: *const CScalarField) -> *mut CScalarField;
    pub fn ScalarField_hCreateFromVoxels(voxels: *const CVoxels) -> *mut CScalarField;
    pub fn ScalarField_hBuildFromVoxels(
        voxels: *const CVoxels,
        scalar_value: c_float,
        sd_threshold: c_float,
    ) -> *mut CScalarField;
    pub fn ScalarField_Destroy(handle: *mut CScalarField);
    pub fn ScalarField_bIsValid(handle: *const CScalarField) -> bool;
    pub fn ScalarField_SetValue(
        this: *mut CScalarField,
        position: *const crate::types::Vector3f,
        value: c_float,
    );
    pub fn ScalarField_bGetValue(
        this: *const CScalarField,
        position: *const crate::types::Vector3f,
        value: *mut c_float,
    ) -> bool;
    pub fn ScalarField_RemoveValue(
        this: *mut CScalarField,
        position: *const crate::types::Vector3f,
    ) -> bool;
    pub fn ScalarField_GetVoxelDimensions(
        this: *const CScalarField,
        x_origin: *mut c_int,
        y_origin: *mut c_int,
        z_origin: *mut c_int,
        x_size: *mut c_int,
        y_size: *mut c_int,
        z_size: *mut c_int,
    );
    pub fn ScalarField_GetSlice(this: *const CScalarField, z_slice: c_int, buffer: *mut c_float);
    pub fn ScalarField_TraverseActive(
        this: *const CScalarField,
        callback: ScalarFieldTraverseCallback,
    );

    // ========================================
    // VectorField functions
    // ========================================

    pub fn VectorField_hCreate() -> *mut CVectorField;
    pub fn VectorField_hCreateCopy(source: *const CVectorField) -> *mut CVectorField;
    pub fn VectorField_hCreateFromVoxels(voxels: *const CVoxels) -> *mut CVectorField;
    pub fn VectorField_hBuildFromVoxels(
        voxels: *const CVoxels,
        value: *const crate::types::Vector3f,
        sd_threshold: c_float,
    ) -> *mut CVectorField;
    pub fn VectorField_Destroy(handle: *mut CVectorField);
    pub fn VectorField_bIsValid(handle: *const CVectorField) -> bool;
    pub fn VectorField_SetValue(
        this: *mut CVectorField,
        position: *const crate::types::Vector3f,
        value: *const crate::types::Vector3f,
    );
    pub fn VectorField_bGetValue(
        this: *const CVectorField,
        position: *const crate::types::Vector3f,
        value: *mut crate::types::Vector3f,
    ) -> bool;
    pub fn VectorField_RemoveValue(
        this: *mut CVectorField,
        position: *const crate::types::Vector3f,
    ) -> bool;
    pub fn VectorField_TraverseActive(
        this: *const CVectorField,
        callback: VectorFieldTraverseCallback,
    );

    // ========================================
    // OpenVdbFile functions
    // ========================================

    pub fn VdbFile_hCreate() -> *mut CVdbFile;
    pub fn VdbFile_hCreateFromFile(file_path: *const c_char) -> *mut CVdbFile;
    pub fn VdbFile_Destroy(handle: *mut CVdbFile);
    pub fn VdbFile_bIsValid(handle: *const CVdbFile) -> bool;
    pub fn VdbFile_bSaveToFile(handle: *const CVdbFile, file_path: *const c_char) -> bool;

    pub fn VdbFile_nFieldCount(handle: *const CVdbFile) -> c_int;
    pub fn VdbFile_nFieldType(handle: *const CVdbFile, index: c_int) -> c_int;
    pub fn VdbFile_GetFieldName(handle: *const CVdbFile, index: c_int, buffer: *mut c_char);

    pub fn VdbFile_hGetVoxels(handle: *const CVdbFile, index: c_int) -> *mut CVoxels;
    pub fn VdbFile_nAddVoxels(
        handle: *mut CVdbFile,
        field_name: *const c_char,
        voxels: *const CVoxels,
    ) -> c_int;

    pub fn VdbFile_hGetScalarField(handle: *const CVdbFile, index: c_int) -> *mut CScalarField;
    pub fn VdbFile_nAddScalarField(
        handle: *mut CVdbFile,
        field_name: *const c_char,
        field: *const CScalarField,
    ) -> c_int;

    pub fn VdbFile_hGetVectorField(handle: *const CVdbFile, index: c_int) -> *mut CVectorField;
    pub fn VdbFile_nAddVectorField(
        handle: *mut CVdbFile,
        field_name: *const c_char,
        field: *const CVectorField,
    ) -> c_int;

    // ========================================
    // Viewer functions
    // ========================================

    pub fn Viewer_hCreate(
        title: *const c_char,
        size: *const crate::types::Vector2f,
        info_cb: ViewerInfoCallback,
        update_cb: ViewerUpdateCallback,
        key_cb: ViewerKeyPressedCallback,
        mouse_move_cb: ViewerMouseMovedCallback,
        mouse_button_cb: ViewerMouseButtonCallback,
        scroll_cb: ViewerScrollWheelCallback,
        window_size_cb: ViewerWindowSizeCallback,
    ) -> *mut CViewer;
    pub fn Viewer_bIsValid(handle: *const CViewer) -> bool;
    pub fn Viewer_Destroy(handle: *mut CViewer);
    pub fn Viewer_RequestUpdate(handle: *mut CViewer);
    pub fn Viewer_bPoll(handle: *mut CViewer) -> bool;
    pub fn Viewer_RequestScreenShot(handle: *mut CViewer, path: *const c_char) -> bool;
    pub fn Viewer_bLoadLightSetup(
        handle: *mut CViewer,
        diffuse_buffer: *const u8,
        diffuse_size: c_int,
        specular_buffer: *const u8,
        specular_size: c_int,
    ) -> bool;
    pub fn Viewer_RequestClose(handle: *mut CViewer);
    pub fn Viewer_AddMesh(handle: *mut CViewer, group_id: c_int, mesh: *mut CMesh);
    pub fn Viewer_RemoveMesh(handle: *mut CViewer, mesh: *mut CMesh);
    pub fn Viewer_AddPolyLine(handle: *mut CViewer, group_id: c_int, polyline: *mut CPolyLine);
    pub fn Viewer_RemovePolyLine(handle: *mut CViewer, polyline: *mut CPolyLine);
    pub fn Viewer_SetGroupVisible(handle: *mut CViewer, group_id: c_int, visible: bool);
    pub fn Viewer_SetGroupStatic(handle: *mut CViewer, group_id: c_int, is_static: bool);
    pub fn Viewer_SetGroupMaterial(
        handle: *mut CViewer,
        group_id: c_int,
        color: *const crate::types::ColorFloat,
        metallic: c_float,
        roughness: c_float,
    );
    pub fn Viewer_SetGroupMatrix(
        handle: *mut CViewer,
        group_id: c_int,
        matrix: *const crate::types::Matrix4x4,
    );

    // ========================================
    // Metadata functions
    // ========================================

    pub fn Metadata_hFromVoxels(voxels: *const CVoxels) -> *mut CMetadata;
    pub fn Metadata_hFromScalarField(field: *const CScalarField) -> *mut CMetadata;
    pub fn Metadata_hFromVectorField(field: *const CVectorField) -> *mut CMetadata;
    pub fn Metadata_Destroy(handle: *mut CMetadata);
    pub fn Metadata_nCount(handle: *const CMetadata) -> c_int;
    pub fn Metadata_nNameLengthAt(handle: *const CMetadata, index: c_int) -> c_int;
    pub fn Metadata_bGetNameAt(
        handle: *const CMetadata,
        index: c_int,
        buffer: *mut c_char,
        max_len: c_int,
    ) -> bool;
    pub fn Metadata_nTypeAt(handle: *const CMetadata, name: *const c_char) -> c_int;
    pub fn Metadata_nStringLengthAt(handle: *const CMetadata, name: *const c_char) -> c_int;
    pub fn Metadata_bGetStringAt(
        handle: *const CMetadata,
        name: *const c_char,
        buffer: *mut c_char,
        max_len: c_int,
    ) -> bool;
    pub fn Metadata_bGetFloatAt(
        handle: *const CMetadata,
        name: *const c_char,
        value: *mut c_float,
    ) -> bool;
    pub fn Metadata_bGetVectorAt(
        handle: *const CMetadata,
        name: *const c_char,
        value: *mut crate::types::Vector3f,
    ) -> bool;
    pub fn Metadata_SetStringValue(
        handle: *mut CMetadata,
        name: *const c_char,
        value: *const c_char,
    );
    pub fn Metadata_SetFloatValue(handle: *mut CMetadata, name: *const c_char, value: c_float);
    pub fn Metadata_SetVectorValue(
        handle: *mut CMetadata,
        name: *const c_char,
        value: *const crate::types::Vector3f,
    );
    #[link_name = "MetaData_RemoveValue"]
    pub fn MetaData_RemoveValue(handle: *mut CMetadata, name: *const c_char);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opaque_types() {
        // Just ensure types compile
        let _: *mut CVoxels = std::ptr::null_mut();
        let _: *mut CMesh = std::ptr::null_mut();
        let _: *mut CLattice = std::ptr::null_mut();
    }
}
