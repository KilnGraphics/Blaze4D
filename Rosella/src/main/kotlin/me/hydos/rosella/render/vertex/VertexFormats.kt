package me.hydos.rosella.render.vertex

class VertexFormats {
	companion object {
		val POSITION = create(of(VertexFormat.Element.POSITION, 0))
		val POSITION_COLOR = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR, 1))
		val POSITION_COLOR_UV = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR, 1), of(VertexFormat.Element.UVf, 2))
		val POSITION_UV = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.UVf, 1))
		val POSITION_UV_COLOR = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.UVf, 1), of(VertexFormat.Element.COLOR, 2))
		val POSITION_COLOR_NORMAL_PADDING = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR4, 1), of(VertexFormat.Element.NORMAL, 2), of(VertexFormat.Element.PADDING, 3))
		val POSITION_COLOR4_UV0_UV = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR4, 1), of(VertexFormat.Element.UVf, 2), of(VertexFormat.Element.UVs, 3))
		val POSITION_COLOR4 = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR4, 1))
		val POSITION_COLOR4_UV = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR4, 1), of(VertexFormat.Element.UVf, 2))
		val POSITION_UV_COLOR4 = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.UVf, 1), of(VertexFormat.Element.COLOR4, 2))
		val POSITION_COLOR4_UV_LIGHT = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR4, 1), of(VertexFormat.Element.UVf, 2), of(VertexFormat.Element.LIGHT, 3))

		private fun of(element: VertexFormat.Element, layoutLoc: Int): Pair<VertexFormat.Element, Int> {
			return Pair(element, layoutLoc)
		}

		private fun create(vararg elementInfo: Pair<VertexFormat.Element, Int>): VertexFormat {
			val map = HashMap<Int, VertexFormat.Element>()
			for (pair in elementInfo) {
				map[pair.second] = pair.first
			}
			return VertexFormat(map)
		}
	}
}