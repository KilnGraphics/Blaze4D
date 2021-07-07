package me.hydos.rosella.render.material.state

data class StateInfo (
    var colorMask: Int,
    var depthMask: Boolean,

    var scissorEnabled: Boolean,
    var scissorX: Int,
    var scissorY: Int,
    var scissorWidth: Int,
    var scissorHeight: Int,

    var stencilEnabled: Boolean,

    var blendEnabled: Boolean,
    var srcColorBlendFactor: Int,
    var dstColorBlendFactor: Int,
    var srcAlphaBlendFactor: Int,
    var dstAlphaBlendFactor: Int,
    var blendOp: Int,

    var cullEnabled: Boolean,

    var depthTestEnabled: Boolean,
    var depthCompareOp: Int,

    var colorLogicOpEnabled: Boolean,
    var colorLogicOp: Int,

    var lineWidth: Float,
) {
    /**
     * @return a deep copy of the contents of this class
     */
    fun snapshot(): StateInfo {
        return copy()
    }
}
