package me.hydos.rosella.render.vertex

class VertexFormats {
	companion object {
		val POSITION = create(of(VertexFormat.Element.POSITION, 0))
		val POSITION_COLOR = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR, 1))
		val POSITION_COLOR_UV = create(of(VertexFormat.Element.POSITION, 0), of(VertexFormat.Element.COLOR, 1), of(VertexFormat.Element.UV, 2))

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