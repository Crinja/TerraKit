mod common;

use std::ffi::c_void;
use std::ptr::NonNull;

use common::{
    capability_id, hydraulic_schema, metadata_key_id, resource_registry, resource_type_id,
    vector_field_schema,
};
use terrakit_core::resource::{
    CapabilityBinding, MetadataValue, ResolvedResourceView, Resource, ResourceAccessError,
    ResourceDescriptor, ResourceError, ResourceId, ResourceMetadata, ResourcePath,
    ResourceRegistry, ResourceStorage, ResourceStorageWriter, ResourceTypeId, ResourceTypeSpec,
    ResourceView, ResourceViewResolveError, Schema, StorageAccessGuard, StorageAccessId,
    StorageAccessInterface, StorageAccessRequest, StorageAccessSupport, StorageError,
    StorageViewState, StorageViewStateError, ViewError,
};

fn test_access_id() -> StorageAccessId {
    StorageAccessId::new("plugin.test.resource-read@1").unwrap()
}

fn other_access_id() -> StorageAccessId {
    StorageAccessId::new("plugin.test.other-read@1").unwrap()
}

static TEST_VTABLE: u8 = 0;

#[test]
fn storage_access_interface_exposes_bound_access_id() {
    let access = test_access_id();
    let vtable = NonNull::from(&TEST_VTABLE).cast::<c_void>();

    // SAFETY: TEST_VTABLE has static lifetime and this test does not interpret
    // the fake contract-specific interface.
    let interface =
        unsafe { StorageAccessInterface::from_raw_parts(&access, std::ptr::null_mut(), vtable) };

    assert_eq!(interface.access_id(), &access);
}

#[test]
fn storage_access_request_derives_access_id_from_interface() {
    let access = test_access_id();
    let vtable = NonNull::from(&TEST_VTABLE).cast::<c_void>();

    // SAFETY: TEST_VTABLE has static lifetime and this test does not interpret
    // the fake contract-specific interface.
    let interface =
        unsafe { StorageAccessInterface::from_raw_parts(&access, std::ptr::null_mut(), vtable) };

    let request = StorageAccessRequest::with_interface(interface);

    assert_eq!(request.access_id(), &access);
    assert_eq!(request.interface().unwrap().access_id(), &access);
}

struct FakeAccessGuard {
    access: StorageAccessId,
}

impl StorageAccessGuard for FakeAccessGuard {
    fn interface(&self) -> StorageAccessInterface<'_> {
        let vtable = NonNull::from(&TEST_VTABLE).cast::<c_void>();

        // SAFETY: TEST_VTABLE has static lifetime and this fake interface uses
        // no context. Its raw parts implement the fake contract identified by
        // self.access for the lifetime of this guard.
        unsafe {
            StorageAccessInterface::from_raw_parts(&self.access, std::ptr::null_mut(), vtable)
        }
    }
}

struct FakeStorage {
    schema: Schema,
    supported_access: Option<StorageAccessId>,
    returned_access: Option<StorageAccessId>,
}

impl FakeStorage {
    fn new(schema: Schema) -> Self {
        Self {
            schema,
            supported_access: None,
            returned_access: None,
        }
    }

    fn with_access(schema: Schema, access: StorageAccessId) -> Self {
        Self {
            schema,
            supported_access: Some(access.clone()),
            returned_access: Some(access),
        }
    }

    fn with_mismatched_access(
        schema: Schema,
        supported_access: StorageAccessId,
        returned_access: StorageAccessId,
    ) -> Self {
        Self {
            schema,
            supported_access: Some(supported_access),
            returned_access: Some(returned_access),
        }
    }

    fn supports(&self, view: &ResolvedResourceView, access: &StorageAccessId) -> bool {
        view.resource_schema() == &self.schema
            && self.supported_access.as_ref() == Some(access)
            && view.schema() == &vector_field_schema()
    }
}

impl ResourceStorage for FakeStorage {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn view_state(&self, view: &ResolvedResourceView) -> Result<StorageViewState, StorageError> {
        if view.resource_schema() != &self.schema {
            return Err(StorageError::ViewSchemaMismatch);
        }

        match view.schema() {
            Schema::DenseArray { rank, .. } => Ok(StorageViewState::DenseArray {
                extents: vec![1; *rank as usize].into_boxed_slice(),
            }),
            _ => Ok(StorageViewState::Static),
        }
    }

    fn access_support(
        &self,
        view: &ResolvedResourceView,
        access: &StorageAccessId,
    ) -> StorageAccessSupport {
        if self.supports(view, access) {
            StorageAccessSupport::Supported
        } else {
            StorageAccessSupport::Unsupported
        }
    }

    fn open_access<'a>(
        &'a self,
        view: &ResolvedResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<Box<dyn StorageAccessGuard + 'a>, StorageError> {
        if !self.supports(view, request.access_id()) {
            return Err(StorageError::UnsupportedAccess(request.access_id().clone()));
        }

        Ok(Box::new(FakeAccessGuard {
            access: self
                .returned_access
                .clone()
                .expect("supported fake access must define returned access"),
        }))
    }
}

struct InvalidStateStorage {
    schema: Schema,
}

impl ResourceStorage for InvalidStateStorage {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn view_state(&self, view: &ResolvedResourceView) -> Result<StorageViewState, StorageError> {
        if view.resource_schema() != &self.schema {
            return Err(StorageError::ViewSchemaMismatch);
        }

        Ok(StorageViewState::DenseArray {
            extents: vec![1].into_boxed_slice(),
        })
    }

    fn access_support(
        &self,
        _view: &ResolvedResourceView,
        _access: &StorageAccessId,
    ) -> StorageAccessSupport {
        StorageAccessSupport::Unsupported
    }

    fn open_access<'a>(
        &'a self,
        _view: &ResolvedResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<Box<dyn StorageAccessGuard + 'a>, StorageError> {
        Err(StorageError::UnsupportedAccess(request.access_id().clone()))
    }
}

struct FakeWriter {
    storage: FakeStorage,
    complete: bool,
}

impl FakeWriter {
    fn new(schema: Schema, complete: bool) -> Self {
        Self {
            storage: FakeStorage::new(schema),
            complete,
        }
    }

    fn with_access(schema: Schema, access: StorageAccessId, complete: bool) -> Self {
        Self {
            storage: FakeStorage::with_access(schema, access),
            complete,
        }
    }
}

impl ResourceStorageWriter for FakeWriter {
    fn schema(&self) -> &Schema {
        self.storage.schema()
    }

    fn view_state(&self, view: &ResolvedResourceView) -> Result<StorageViewState, StorageError> {
        self.storage.view_state(view)
    }

    fn access_support(
        &self,
        view: &ResolvedResourceView,
        access: &StorageAccessId,
    ) -> StorageAccessSupport {
        self.storage.access_support(view, access)
    }

    fn open_access<'a>(
        &'a mut self,
        view: &ResolvedResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<Box<dyn StorageAccessGuard + 'a>, StorageError> {
        self.storage.open_access(view, request)
    }

    fn finish(self: Box<Self>) -> Result<Box<dyn ResourceStorage>, StorageError> {
        if !self.complete {
            return Err(StorageError::Incomplete);
        }

        Ok(Box::new(self.storage))
    }
}

fn hydraulic_registry() -> (ResourceRegistry, ResourceTypeId) {
    let mut registry = resource_registry();
    let resource_type = resource_type_id("domain.hydraulic-erosion-result@1");

    registry
        .register_resource_type(
            ResourceTypeSpec::new(resource_type.clone(), hydraulic_schema()).with_capability(
                CapabilityBinding::view(
                    capability_id("terrakit.erosion-flow@1"),
                    ResourceView::Path(ResourcePath::field("flow")),
                ),
            ),
        )
        .unwrap();

    (registry, resource_type)
}

fn hydraulic_descriptor(
    registry: &ResourceRegistry,
    resource_type: ResourceTypeId,
) -> ResourceDescriptor {
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        metadata_key_id("terrakit.coordinate-space@1"),
        MetadataValue::Identifier("world".into()),
    );

    ResourceDescriptor::new(ResourceId(42), resource_type, metadata, registry).unwrap()
}

fn hydraulic_resource(registry: &ResourceRegistry, resource_type: ResourceTypeId) -> Resource {
    Resource::new(
        hydraulic_descriptor(registry, resource_type),
        Box::new(FakeStorage::new(hydraulic_schema())),
        registry,
    )
    .unwrap()
}

#[test]
fn resource_accepts_storage_with_matching_schema() {
    let (registry, resource_type) = hydraulic_registry();
    let resource = hydraulic_resource(&registry, resource_type);

    assert_eq!(resource.id(), &ResourceId(42));
    assert_eq!(resource.schema(), &hydraulic_schema());
}

#[test]
fn resource_rejects_storage_with_different_schema() {
    let (registry, resource_type) = hydraulic_registry();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let storage = Box::new(FakeStorage::new(vector_field_schema()));

    assert!(matches!(
        Resource::new(descriptor, storage, &registry),
        Err(ResourceError::StorageSchemaMismatch { expected, actual })
            if expected == hydraulic_schema() && actual == vector_field_schema()
    ));
}

#[test]
fn resource_rejects_descriptor_from_another_registry() {
    let (registry_a, resource_type_a) = hydraulic_registry();
    let descriptor = hydraulic_descriptor(&registry_a, resource_type_a);
    let (registry_b, _) = hydraulic_registry();
    let storage = Box::new(FakeStorage::new(hydraulic_schema()));

    assert!(matches!(
        Resource::new(descriptor, storage, &registry_b),
        Err(ResourceError::ForeignDescriptor)
    ));
}

#[test]
fn resolved_view_can_be_created_from_schema_without_resource() {
    let flow = ResourceView::Path(ResourcePath::field("flow"));
    let resolved = ResolvedResourceView::resolve(&flow, &hydraulic_schema()).unwrap();

    assert_eq!(resolved.view(), &flow);
    assert_eq!(resolved.resource_schema(), &hydraulic_schema());
    assert_eq!(resolved.schema(), &vector_field_schema());
}

#[test]
fn resolved_view_preserves_typed_view_error() {
    let invalid = ResourceView::Path(ResourcePath::field("missing"));

    assert!(matches!(
        ResolvedResourceView::resolve(&invalid, &hydraulic_schema()),
        Err(ResourceViewResolveError::InvalidView {
            view,
            error: ViewError::UnknownField(field),
        }) if view == invalid && field.as_ref() == "missing"
    ));
}

#[test]
fn writer_can_resolve_then_open_view_without_resource() {
    let access = test_access_id();
    let mut writer: Box<dyn ResourceStorageWriter> = Box::new(FakeWriter::with_access(
        hydraulic_schema(),
        access.clone(),
        true,
    ));
    let flow = ResourceView::Path(ResourcePath::field("flow"));
    let resolved = ResolvedResourceView::resolve(&flow, writer.schema()).unwrap();
    let request = StorageAccessRequest::new(&access);

    assert_eq!(
        writer.access_support(&resolved, &access),
        StorageAccessSupport::Supported
    );

    let guard = writer.open_access(&resolved, &request).unwrap();
    assert_eq!(guard.interface().access_id(), &access);
}

#[test]
fn constructed_resource_resolves_views_without_registry() {
    let (registry, resource_type) = hydraulic_registry();
    let resource = hydraulic_resource(&registry, resource_type);
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    let resolved = resource.resolve_view(&flow).unwrap();

    assert_eq!(resolved.view(), &flow);
    assert_eq!(resolved.resource_schema(), resource.schema());
    assert_eq!(resolved.schema(), &vector_field_schema());
}

#[test]
fn constructed_resource_rejects_invalid_view_without_registry() {
    let (registry, resource_type) = hydraulic_registry();
    let resource = hydraulic_resource(&registry, resource_type);
    let invalid = ResourceView::Path(ResourcePath::field("missing"));

    assert!(matches!(
        resource.resolve_view(&invalid),
        Err(ResourceViewResolveError::InvalidView { view, .. }) if view == invalid
    ));
}

#[test]
fn resource_validates_runtime_structural_state() {
    let (registry, resource_type) = hydraulic_registry();
    let resource = hydraulic_resource(&registry, resource_type);
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert_eq!(
        resource.view_state(&flow).unwrap(),
        StorageViewState::DenseArray {
            extents: vec![1, 1].into_boxed_slice(),
        }
    );
}

#[test]
fn resource_rejects_runtime_state_that_violates_schema() {
    let (registry, resource_type) = hydraulic_registry();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(InvalidStateStorage {
            schema: hydraulic_schema(),
        }),
        &registry,
    )
    .unwrap();
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert!(matches!(
        resource.view_state(&flow),
        Err(ResourceAccessError::InvalidStorageState(
            StorageViewStateError::DenseArrayRankMismatch {
                expected: 2,
                actual: 1,
            }
        ))
    ));
}

#[test]
fn access_contracts_are_extensible_versioned_ids() {
    let (registry, resource_type) = hydraulic_registry();
    let access = test_access_id();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(FakeStorage::with_access(hydraulic_schema(), access.clone())),
        &registry,
    )
    .unwrap();
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert_eq!(
        resource.access_support(&flow, &access).unwrap(),
        StorageAccessSupport::Supported
    );

    let other = other_access_id();
    assert_eq!(
        resource.access_support(&flow, &other).unwrap(),
        StorageAccessSupport::Unsupported
    );
}

#[test]
fn resource_opens_access_through_core_boundary() {
    let (registry, resource_type) = hydraulic_registry();
    let access = test_access_id();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(FakeStorage::with_access(hydraulic_schema(), access.clone())),
        &registry,
    )
    .unwrap();
    let flow = ResourceView::Path(ResourcePath::field("flow"));
    let request = StorageAccessRequest::new(&access);

    let opened = resource.open_access(&flow, &request).unwrap();

    assert_eq!(opened.interface().access_id(), &access);
    assert_eq!(opened.view().schema(), &vector_field_schema());
    assert_eq!(
        opened.state(),
        &StorageViewState::DenseArray {
            extents: vec![1, 1].into_boxed_slice(),
        }
    );

    let interface = opened.interface();
    assert_eq!(interface.access_id(), &access);
    assert_eq!(
        interface.vtable(),
        NonNull::from(&TEST_VTABLE).cast::<c_void>()
    );
}

#[test]
fn invalid_runtime_state_is_rejected_before_access_is_exposed() {
    let (registry, resource_type) = hydraulic_registry();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(InvalidStateStorage {
            schema: hydraulic_schema(),
        }),
        &registry,
    )
    .unwrap();
    let access = test_access_id();
    let request = StorageAccessRequest::new(&access);
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert!(matches!(
        resource.open_access(&flow, &request),
        Err(ResourceAccessError::InvalidStorageState(
            StorageViewStateError::DenseArrayRankMismatch {
                expected: 2,
                actual: 1,
            }
        ))
    ));
}

#[test]
fn resource_rejects_unadvertised_access_before_opening() {
    let (registry, resource_type) = hydraulic_registry();
    let resource = hydraulic_resource(&registry, resource_type);
    let access = test_access_id();
    let request = StorageAccessRequest::new(&access);
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert!(matches!(
        resource.open_access(&flow, &request),
        Err(ResourceAccessError::UnsupportedAccess(id)) if id == access
    ));
}

#[test]
fn resource_rejects_guard_for_wrong_access_contract() {
    let (registry, resource_type) = hydraulic_registry();
    let requested = test_access_id();
    let returned = other_access_id();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(FakeStorage::with_mismatched_access(
            hydraulic_schema(),
            requested.clone(),
            returned.clone(),
        )),
        &registry,
    )
    .unwrap();
    let flow = ResourceView::Path(ResourcePath::field("flow"));
    let request = StorageAccessRequest::new(&requested);

    assert!(matches!(
        resource.open_access(&flow, &request),
        Err(ResourceAccessError::AccessContractMismatch {
            requested: actual_requested,
            returned: actual_returned,
        }) if actual_requested == requested && actual_returned == returned
    ));
}

#[test]
fn access_support_is_view_specific() {
    let (registry, resource_type) = hydraulic_registry();
    let access = test_access_id();
    let descriptor = hydraulic_descriptor(&registry, resource_type);
    let resource = Resource::new(
        descriptor,
        Box::new(FakeStorage::with_access(hydraulic_schema(), access.clone())),
        &registry,
    )
    .unwrap();

    assert_eq!(
        resource
            .access_support(&ResourceView::Root, &access)
            .unwrap(),
        StorageAccessSupport::Unsupported
    );
}

#[test]
fn storage_writer_can_be_finished_through_erased_runtime_interface() {
    let incomplete: Box<dyn ResourceStorageWriter> =
        Box::new(FakeWriter::new(hydraulic_schema(), false));
    let complete: Box<dyn ResourceStorageWriter> =
        Box::new(FakeWriter::new(hydraulic_schema(), true));

    assert_eq!(incomplete.schema(), &hydraulic_schema());
    assert!(matches!(incomplete.finish(), Err(StorageError::Incomplete)));

    let storage = complete.finish().unwrap();
    assert_eq!(storage.schema(), &hydraulic_schema());
}

#[test]
fn static_state_only_describes_selected_view() {
    let schema = Schema::structure(vec![
        terrakit_core::resource::SchemaField::new(
            "values",
            Schema::Array(Box::new(Schema::Numeric(
                terrakit_core::resource::NumericType::F32,
            ))),
        )
        .unwrap(),
    ])
    .unwrap();

    StorageViewState::Static.validate_for(&schema).unwrap();

    let values = ResourceView::Path(ResourcePath::field("values"));
    let resolved = ResolvedResourceView::resolve(&values, &schema).unwrap();
    StorageViewState::Array { length: 42 }
        .validate_for(resolved.schema())
        .unwrap();
}

#[test]
fn published_resources_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Resource>();
}

#[test]
fn storage_writer_is_send() {
    fn assert_send<T: Send>() {}

    assert_send::<FakeWriter>();
}
