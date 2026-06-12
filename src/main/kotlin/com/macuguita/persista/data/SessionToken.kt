package com.macuguita.persista.data

import java.util.UUID

data class SessionToken(
	val userId: UUID,
	val sessionToken: String,
	val expiresAt: String,
)
