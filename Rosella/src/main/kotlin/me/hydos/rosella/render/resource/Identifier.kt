package me.hydos.rosella.render.resource

data class Identifier(val namespace: String, val path: String) {

    val file: String = "$namespace/$path"

    override fun toString(): String {
        return "$namespace:$path"
    }

    companion object {
        @JvmStatic
        val EMPTY = Identifier("rosella", "empty")
    }
}
