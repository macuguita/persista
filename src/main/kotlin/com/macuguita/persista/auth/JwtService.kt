package com.macuguita.persista.auth

import com.auth0.jwt.JWT
import com.auth0.jwt.JWTVerifier
import com.auth0.jwt.algorithms.Algorithm
import com.macuguita.persista.data.SessionToken
import java.util.Date
import java.util.UUID

object JwtService {
	private const val ISSUER = "persista"

	private const val TTL_MS = 24 * 60 * 60 * 1000L

	fun mint(
		secret: String,
		userId: UUID,
	): SessionToken {
		val expiresAt = Date(System.currentTimeMillis() + TTL_MS)
		val token =
			JWT
				.create()
				.withIssuer(ISSUER)
				.withClaim("user_id", userId.toString())
				.withExpiresAt(expiresAt)
				.sign(Algorithm.HMAC256(secret))
		return SessionToken(
			userId = userId,
			sessionToken = token,
			expiresAt = expiresAt.toInstant().toString(),
		)
	}

	fun verifier(secret: String): JWTVerifier =
		JWT
			.require(Algorithm.HMAC256(secret))
			.withIssuer(ISSUER)
			.build()
}
