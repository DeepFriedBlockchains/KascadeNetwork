pub mod channel;
pub mod convert;
pub mod ext;
pub mod macros;
pub mod ops;

/// Maximum decoded gRPC message size to send and receive
pub const RPC_MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GB

// Generated from `proto/messages.proto`. `Payload` is a one-of over every RPC message,
// so its largest variant (SubmitBlockRequest) dwarfs the rest by design; boxing it would
// change the generated wire API. Scope the size lint to this generated module only.
#[allow(clippy::large_enum_variant)]
pub mod protowire {
    tonic::include_proto!("protowire");
}
