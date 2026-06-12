package com.macuguita.persista.auth

import com.macuguita.persista.data.network.MojangProfile
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.HttpStatusCode
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import java.security.SecureRandom

object MojangAuth {
	private val httpClient =
		HttpClient(CIO) {
			install(ContentNegotiation) { json(Json { ignoreUnknownKeys = true }) }
		}

	// In-memory challenge store: playerUUID -> (challenge, expiresAtMs)
	// Challenges are only valid for 30 seconds, so this stays tiny.
	private val challenges = mutableMapOf<String, Pair<String, Long>>()
	private val mutex = Mutex()

	private val rng = SecureRandom()

	fun generateChallenge(): String {
		val bytes = ByteArray(20)
		rng.nextBytes(bytes)
		// Mojang's serverId field must be a hex string
		return bytes.joinToString("") { "%02x".format(it) }
	}

	suspend fun storeChallenge(
		playerId: String,
		challenge: String,
	) {
		mutex.withLock {
			purgeExpired()
			challenges[playerId] = challenge to (System.currentTimeMillis() + 30_000L)
		}
	}

	suspend fun consumeChallenge(playerId: String): String? {
		return mutex.withLock {
			val (challenge, expiresAt) = challenges[playerId] ?: return@withLock null
			challenges.remove(playerId)
			if (System.currentTimeMillis() > expiresAt) null else challenge
		}
	}

	// Calls Mojang to verify the client actually authenticated with the challenge.
	// Returns the player's profile if valid, null if not.
	suspend fun verifyWithMojang(
		username: String,
		challenge: String,
	): MojangProfile? {
		val response =
			httpClient.get("https://sessionserver.mojang.com/session/minecraft/hasJoined") {
				parameter("username", username)
				parameter("serverId", challenge)
			}
		if (response.status != HttpStatusCode.OK) return null
		return response.body<MojangProfile>()
	}

	private fun purgeExpired() {
		val now = System.currentTimeMillis()
		challenges.entries.removeIf { (_, v) -> v.second < now }
	}
}
