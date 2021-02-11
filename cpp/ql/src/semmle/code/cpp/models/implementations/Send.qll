/**
 * Provides implementation classes modeling `send` and various similar
 * functions. See `semmle.code.cpp.models.Models` for usage information.
 */

import semmle.code.cpp.models.interfaces.Taint
import semmle.code.cpp.models.interfaces.ArrayFunction
import semmle.code.cpp.models.interfaces.Alias
import semmle.code.cpp.models.interfaces.FlowSource
import semmle.code.cpp.models.interfaces.SideEffect

/** The function `send` and its assorted variants */
private class Send extends AliasFunction, ArrayFunction, SideEffectFunction, RemoteFlowFunctionSink {
  Send() {
    this.hasGlobalName([
        "send", // send(socket, buf, len, flags)
        "sendto", // sendto(socket, buf, len, flags, to, tolen)
        "write" // write(socket, buf, len);
      ])
  }

  override predicate parameterNeverEscapes(int index) {
    this.getParameter(index).getUnspecifiedType() instanceof PointerType
  }

  override predicate parameterEscapesOnlyViaReturn(int index) { none() }

  override predicate parameterIsAlwaysReturned(int index) { none() }

  override predicate hasArrayWithVariableSize(int bufParam, int countParam) {
    bufParam = 1 and countParam = 2
  }

  override predicate hasArrayInput(int bufParam) { bufParam = 1 }

  override predicate hasOnlySpecificReadSideEffects() { any() }

  override predicate hasOnlySpecificWriteSideEffects() { any() }

  override predicate hasSpecificWriteSideEffect(ParameterIndex i, boolean buffer, boolean mustWrite) {
    none()
  }

  override predicate hasSpecificReadSideEffect(ParameterIndex i, boolean buffer) {
    i = 1 and buffer = true
    or
    exists(this.getParameter(4)) and i = 4 and buffer = false
  }

  override ParameterIndex getParameterSizeIndex(ParameterIndex i) { i = 1 and result = 2 }

  override predicate hasRemoteFlowSink(FunctionInput input, string description) {
    input.isParameterDeref(1) and description = "Buffer sent by " + this.getName()
  }
}
