package me.hydos.rosella.render.font

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.resource.Resource
import java.awt.Font

object FontHelper {

	fun loadFont(fontFile: Resource, rosella: Rosella): RosellaFont {
		val font = Font.createFont(Font.TRUETYPE_FONT, fontFile.openStream()).deriveFont(Font.BOLD, 80f)
		return RosellaFont(font, rosella)
	}
}
