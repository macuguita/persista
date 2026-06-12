package com.macuguita.persista.data.network

import kotlinx.serialization.Serializable

@Serializable
data class SessionResponse(
	val user_id: String,
	val session_token: String,
	val expires_at: String,
)
