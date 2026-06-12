package com.macuguita.persista

import com.macuguita.persista.auth.JwtService
import com.macuguita.persista.auth.MojangAuth
import com.macuguita.persista.data.Config
import com.macuguita.persista.data.Identifier
import com.macuguita.persista.data.network.AuthRequest
import com.macuguita.persista.data.network.ChallengeRequest
import com.macuguita.persista.data.network.ChallengeResponse
import com.macuguita.persista.data.network.Entitlements
import com.macuguita.persista.data.network.SessionResponse
import com.macuguita.persista.databse.DatabaseFactory
import com.macuguita.persista.databse.PlayerDataTable
import com.macuguita.persista.databse.json
import io.ktor.http.HttpStatusCode
import io.ktor.serialization.kotlinx.json.json
import io.ktor.server.application.Application
import io.ktor.server.application.ApplicationCall
import io.ktor.server.application.install
import io.ktor.server.auth.Authentication
import io.ktor.server.auth.authenticate
import io.ktor.server.auth.jwt.JWTPrincipal
import io.ktor.server.auth.jwt.jwt
import io.ktor.server.auth.principal
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.plugins.contentnegotiation.ContentNegotiation
import io.ktor.server.plugins.origin
import io.ktor.server.plugins.ratelimit.RateLimit
import io.ktor.server.plugins.ratelimit.RateLimitName
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.plugins.statuspages.StatusPages
import io.ktor.server.request.receive
import io.ktor.server.response.respond
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.route
import io.ktor.server.routing.routing
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement
import org.jetbrains.exposed.v1.core.and
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.jdbc.deleteWhere
import org.jetbrains.exposed.v1.jdbc.selectAll
import org.jetbrains.exposed.v1.jdbc.transactions.transaction
import org.jetbrains.exposed.v1.jdbc.upsert
import org.slf4j.LoggerFactory
import java.util.UUID
import kotlin.time.Duration.Companion.minutes
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

val ENTITLEMENTS_KEY = Identifier("persista", "entitlements")
val LOGGER = LoggerFactory.getLogger("persista-api")

fun main() {
	LOGGER.info("Starting Persista on port {}", Config.port)
	embeddedServer(Netty, port = Config.port, module = Application::module).start(wait = true)
}

@OptIn(ExperimentalUuidApi::class)
fun Application.module() {
	initDatabase()

	install(ContentNegotiation) {
		json()
	}

	install(StatusPages) {
		exception<IllegalArgumentException> { call, cause ->
			call.respond(HttpStatusCode.BadRequest, mapOf("error" to (cause.message ?: "Bad request")))
		}
		exception<UnauthorizedException> { call, cause ->
			call.respond(HttpStatusCode.Unauthorized, mapOf("error" to (cause.message ?: "Unauthorized request")))
		}
		exception<Throwable> { call, cause ->
			LOGGER.error("Unhandled exception", cause)
			call.respond(HttpStatusCode.InternalServerError, mapOf("error" to "internal server error"))
		}
	}

	install(RateLimit) {
		register(RateLimitName("auth")) {
			rateLimiter(limit = 10, refillPeriod = 1.minutes)
			requestKey { call ->
				call.request.headers["X-Forwarded-For"]
					?: call.request.origin.remoteAddress
			}
		}
		register(RateLimitName("data")) {
			rateLimiter(limit = 60, refillPeriod = 1.minutes)
			requestKey { call ->
				call.request.headers["X-Forwarded-For"]
					?: call.request.origin.remoteAddress
			}
		}
	}

	install(Authentication) {
		jwt("jwt") {
			verifier(JwtService.verifier(Config.jwtSecret))
			validate { credential ->
				// The JWT payload must have a "user_id" claim (UUID)
				if (credential.payload.getClaim("user_id").asString() != null) {
					JWTPrincipal(credential.payload)
				} else {
					null
				}
			}
			challenge { _, _ ->
				call.respond(HttpStatusCode.Unauthorized, mapOf("error" to "invalid or missing token"))
			}
		}
	}

	routing {
		rateLimit(RateLimitName("auth")) {
			route("/api/auth/mojang") {
				// Step 1: Client asks for a challenge token
				post("/challenge") {
					val body = call.receive<ChallengeRequest>()
					val playerId = body.id.ifBlank { throw IllegalArgumentException("id is required") }

					val challenge = MojangAuth.generateChallenge()
					MojangAuth.storeChallenge(playerId, challenge)

					call.respond(ChallengeResponse(token = challenge))
				}
				// Step 2: Client has called joinServer() with the challenge,
				// now sends back their username + challenge for us to verify with Mojang.
				post {
					val body = call.receive<AuthRequest>()

					val storedChallenge =
						MojangAuth.consumeChallenge(body.id)
							?: throw UnauthorizedException("challenge not found or expired")

					if (storedChallenge != body.token) {
						throw UnauthorizedException("challenge token mismatch")
					}

					val profile =
						MojangAuth.verifyWithMojang(body.username, storedChallenge)
							?: run {
								LOGGER.warn("Mojang verification failed for player {} ({})", body.username, body.id)
								throw UnauthorizedException("Mojang verification failed")
							}

					// Sanity check: the UUID Mojang returns must match what the client claimed
					// Mojang returns UUIDs without hyphens, so we normalize
					val mojangId = profile.id.replace("-", "")
					val claimedId = body.id.replace("-", "")
					if (mojangId != claimedId) {
						throw UnauthorizedException("UUID mismatch")
					}

					val userId =
						UUID.fromString(
							profile.id.let {
								if (it.contains('-')) {
									it
								} else {
									"${it.substring(0, 8)}-${it.substring(8, 12)}-${it.substring(12, 16)}" +
										"-${it.substring(16, 20)}-${it.substring(20)}"
								}
							},
						)

					val session = JwtService.mint(Config.jwtSecret, userId)
					LOGGER.info("Player {} authenticated successfully", userId)
					call.respond(
						SessionResponse(
							user_id = session.userId.toString(),
							session_token = session.sessionToken,
							expires_at = session.expiresAt,
						),
					)
				}
			}
		}

		rateLimit(RateLimitName("data")) {
			route("/api/v0/data/{uuid}/{namespace}/{path}") {
				// Public read — anyone can fetch any player's data by UUID.
				get {
					val uuid =
						call.parameters["uuid"]?.toUUIDOrNull()
							?: throw IllegalArgumentException("invalid uuid")
					val namespace = call.parameters["namespace"]!!
					val path = call.parameters["path"]!!

					val json =
						transaction {
							PlayerDataTable
								.selectAll()
								.where {
									(PlayerDataTable.uuid eq uuid) and
										(PlayerDataTable.namespace eq namespace) and
										(PlayerDataTable.path eq path)
								}.singleOrNull()
								?.get(PlayerDataTable.value)
						}

					if (json == null) {
						call.respond(HttpStatusCode.NotFound, mapOf("error" to "no data found"))
						return@get
					}

					call.respond(json)
				}
				// Authenticated write — only the player themselves can update their own data.
				// Exception: entitlements can only be updated via the admin endpoint below.
				authenticate("jwt") {
					post {
						val uuid =
							call.parameters["uuid"]?.toUUIDOrNull()
								?: throw IllegalArgumentException("invalid uuid")

						val namespace = call.parameters["namespace"]!!
						val path = call.parameters["path"]!!
						val dataId =
							try {
								Identifier(namespace, path)
							} catch (_: IllegalArgumentException) {
								throw IllegalArgumentException("invalid identifier: $namespace:$path")
							}

						// Block entitlements table from being modified directly
						if (dataId == ENTITLEMENTS_KEY) {
							throw UnauthorizedException("entitlements are managed server-side only")
						}

						val principal = call.principal<JWTPrincipal>()!!
						val tokenUserId =
							principal.payload.getClaim("user_id").asString()
								?: throw UnauthorizedException("missing user_id")

						if (tokenUserId != uuid.toString()) {
							throw UnauthorizedException("cannot write data for another player")
						}

						val entitlements = fetchEntitlements(uuid)

						if (!entitlements.contains(dataId)) {
							throw UnauthorizedException("missing entitlement: $dataId")
						}

						val body = call.receive<JsonElement>()

						updateDb(uuid, dataId, body)

						call.respond(HttpStatusCode.NoContent)
					}
				}
			}
		}

		route("/api/admin") {
			// Example:
			// ```bash
			// curl -X POST https://persista.macuguita.com/api/admin/entitlements/{uuid} \
			//  -H "Content-Type: application/json" \
			//  -H "X-Admin-Secret: secret" \
			//  -d '{"values": ["macu_lib:supporter"]}'
			// ´´´
			post("/entitlements/{uuid}") {
				// Simple shared-secret admin auth
				call.requireAdminSecret()

				val uuid =
					call.parameters["uuid"]?.toUUIDOrNull()
						?: throw IllegalArgumentException("invalid uuid")

				val body = call.receive<Entitlements>()
				body.values.forEach {
					Identifier.tryParse(it) ?: throw IllegalArgumentException("invalid identifier: $it")
				}

				val entitlements = Json.encodeToJsonElement(body)

				updateDb(uuid, ENTITLEMENTS_KEY, entitlements)
				LOGGER.info("Entitlements updated for {}: {}", uuid, body.values)

				call.respond(HttpStatusCode.OK, mapOf("updated" to uuid.toString()))
			}
			delete("/data/{uuid}") {
				call.requireAdminSecret()

				val uuid =
					call.parameters["uuid"]?.toUUIDOrNull()
						?: throw IllegalArgumentException("invalid uuid")

				val deleted =
					transaction {
						PlayerDataTable.deleteWhere {
							PlayerDataTable.uuid eq uuid
						}
					}

				LOGGER.info("Deleted all data for player {}", uuid)
				call.respond(HttpStatusCode.OK, mapOf("deleted" to uuid.toString(), "rows" to deleted))
			}
		}
	}
}

@OptIn(ExperimentalUuidApi::class)
private fun updateDb(
	uuid: Uuid,
	identifier: Identifier,
	body: JsonElement,
) {
	transaction {
		PlayerDataTable.upsert {
			it[PlayerDataTable.uuid] = uuid
			it[PlayerDataTable.namespace] = identifier.namespace
			it[PlayerDataTable.path] = identifier.path
			it[PlayerDataTable.value] = body
			it[PlayerDataTable.updatedAt] = System.currentTimeMillis()
		}
	}
}

@OptIn(ExperimentalUuidApi::class)
fun fetchEntitlements(uuid: Uuid): Entitlements =
	transaction {
		PlayerDataTable
			.selectAll()
			.where {
				(PlayerDataTable.uuid eq uuid) and
					(PlayerDataTable.namespace eq ENTITLEMENTS_KEY.namespace) and
					(PlayerDataTable.path eq ENTITLEMENTS_KEY.path)
			}.singleOrNull()
			?.get(PlayerDataTable.value)
			?.let { json.decodeFromJsonElement<Entitlements>(it) }
			?: Entitlements.EMPTY
	}

fun ApplicationCall.requireAdminSecret() {
	val provided = request.headers["X-Admin-Secret"]
	if (provided != Config.adminSecret) throw UnauthorizedException("invalid admin secret")
}

fun initDatabase() {
	DatabaseFactory.init(
		url = Config.dbUrl,
		user = Config.dbUser,
		password = Config.dbPass,
	)
}

@OptIn(ExperimentalUuidApi::class)
private fun String.toUUIDOrNull(): Uuid? = Uuid.parseOrNull(this)

class UnauthorizedException(
	message: String,
) : Exception(message)
