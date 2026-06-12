package com.macuguita.persista.data.network

import kotlinx.serialization.Serializable

@Serializable
data class ChallengeResponse(
	val token: String,
	val expiresIn: Int = 30,
)
