use boa_engine::{
    builtins::JsArgs,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    Context, JsNativeError, JsResult, JsValue, Source,
};

/// Initializes the object in the context.
pub(super) fn init(context: &mut Context<'_>) -> JsObject {
    let global_obj = context.global_object().clone();

    let obj = ObjectInitializer::new(context)
        .function(create_realm, "createRealm", 0)
        .function(detach_array_buffer, "detachArrayBuffer", 2)
        .function(eval_script, "evalScript", 1)
        .function(gc, "gc", 0)
        .property("global", global_obj, Attribute::default())
        // .property("agent", agent, Attribute::default())
        .build();

    context.register_global_property("$262", obj.clone(), Attribute::empty());

    obj
}

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
#[allow(clippy::unnecessary_wraps)]
fn create_realm(_this: &JsValue, _: &[JsValue], _context: &mut Context<'_>) -> JsResult<JsValue> {
    let mut context = Context::default();

    // add the $262 object.
    let js_262 = init(&mut context);

    Ok(JsValue::new(js_262))
}

/// The `$262.detachArrayBuffer()` function.
///
/// Implements the `DetachArrayBuffer` abstract operation.
fn detach_array_buffer(
    _this: &JsValue,
    args: &[JsValue],
    _: &mut Context<'_>,
) -> JsResult<JsValue> {
    fn type_err() -> JsNativeError {
        JsNativeError::typ().with_message("The provided object was not an ArrayBuffer")
    }

    let array_buffer = args
        .get(0)
        .and_then(JsValue::as_object)
        .ok_or_else(type_err)?;
    let mut array_buffer = array_buffer.borrow_mut();
    let array_buffer = array_buffer.as_array_buffer_mut().ok_or_else(type_err)?;

    // 1. Assert: IsSharedArrayBuffer(arrayBuffer) is false. TODO
    // 2. If key is not present, set key to undefined.
    let key = args.get_or_undefined(1);

    // 3. If SameValue(arrayBuffer.[[ArrayBufferDetachKey]], key) is false, throw a TypeError exception.
    if !JsValue::same_value(&array_buffer.array_buffer_detach_key, key) {
        return Err(JsNativeError::typ()
            .with_message("Cannot detach array buffer with different key")
            .into());
    }

    // 4. Set arrayBuffer.[[ArrayBufferData]] to null.
    array_buffer.array_buffer_data = None;

    // 5. Set arrayBuffer.[[ArrayBufferByteLength]] to 0.
    array_buffer.array_buffer_byte_length = 0;

    // 6. Return NormalCompletion(null).
    Ok(JsValue::null())
}

/// The `$262.evalScript()` function.
///
/// Accepts a string value as its first argument and executes it as an ECMAScript script.
fn eval_script(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    args.get(0).and_then(JsValue::as_string).map_or_else(
        || Ok(JsValue::undefined()),
        |source_text| match context.parse(Source::from_bytes(&source_text.to_std_string_escaped()))
        {
            // TODO: check strict
            Err(e) => Err(JsNativeError::typ()
                .with_message(format!("Uncaught Syntax Error: {e}"))
                .into()),
            // Calling eval here parses the code a second time.
            // TODO: We can fix this after we have have defined the public api for the vm executer.
            Ok(_) => context.eval(Source::from_bytes(&source_text.to_std_string_escaped())),
        },
    )
}

/// The `$262.gc()` function.
///
/// Wraps the host's garbage collection invocation mechanism, if such a capability exists.
/// Must throw an exception if no capability exists. This is necessary for testing the
/// semantics of any feature that relies on garbage collection, e.g. the `WeakRef` API.
#[allow(clippy::unnecessary_wraps)]
fn gc(_this: &JsValue, _: &[JsValue], _context: &mut Context<'_>) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}
