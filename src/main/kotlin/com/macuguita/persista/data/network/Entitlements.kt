package com.macuguita.persista.data.network

import com.macuguita.persista.data.Identifier
import kotlinx.serialization.Serializable

@Serializable
data class Entitlements(
	val values: List<String>,
) {
	fun toIdentifiers(): List<Identifier> = values.mapNotNull { Identifier.tryParse(it) }

	fun contains(identifier: Identifier): Boolean = identifier in toIdentifiers()

	companion object {
		val EMPTY = Entitlements(emptyList())
	}
}
