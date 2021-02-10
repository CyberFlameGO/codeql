/**
 * Provides classes for modelling common markdown parsers and generators.
 */

import javascript

/**
 * A taint step for the `marked` library, that converts markdown to HTML.
 */
private class MarkedStep extends TaintTracking::AdditionalTaintStep, DataFlow::CallNode {
  MarkedStep() {
    this = DataFlow::globalVarRef("marked").getACall()
    or
    this = DataFlow::moduleImport("marked").getACall()
  }

  override predicate step(DataFlow::Node pred, DataFlow::Node succ) {
    succ = this and
    pred = this.getAnArgument()
  }
}
