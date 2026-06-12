package com.macuguita.persista.data.network

import kotlinx.serialization.Serializable

@Serializable
data class AuthRequest(
	val id: String,
	val username: String,
	val token: String,
)
