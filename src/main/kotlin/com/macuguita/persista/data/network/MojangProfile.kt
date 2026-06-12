package com.macuguita.persista.data.network

import kotlinx.serialization.Serializable

@Serializable
data class MojangProfile(
	val id: String,
	val name: String,
)
