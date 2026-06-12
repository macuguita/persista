package com.macuguita.persista.databse

import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import org.jetbrains.exposed.v1.core.Table
import org.jetbrains.exposed.v1.jdbc.Database
import org.jetbrains.exposed.v1.jdbc.transactions.transaction
import org.jetbrains.exposed.v1.json.jsonb
import org.jetbrains.exposed.v1.migration.jdbc.MigrationUtils
import kotlin.uuid.ExperimentalUuidApi

object DatabaseFactory {
	fun init(
		url: String,
		user: String,
		password: String,
	) {
		val config =
			HikariConfig().apply {
				jdbcUrl = url
				username = user
				this.password = password
				maximumPoolSize = 10
				isAutoCommit = false
				transactionIsolation = "TRANSACTION_REPEATABLE_READ"
				validate()
			}
		Database.connect(HikariDataSource(config))
		runMigrations()
	}

	private fun runMigrations() {
		transaction {
			val statements =
				MigrationUtils.statementsRequiredForDatabaseMigration(
					PlayerDataTable,
					withLogs = true,
				)
			for (statement in statements) {
				exec(statement)
			}
		}
	}
}

val json =
	Json {
		ignoreUnknownKeys = true
		encodeDefaults = true
	}

@OptIn(ExperimentalUuidApi::class)
object PlayerDataTable : Table("player_data") {
	val uuid = uuid("uuid")
	val namespace = varchar("namespace", 64)
	val path = varchar("path", 128)
	val value = jsonb<JsonElement>("value", json) // raw JSON string stored as JSONB
	val updatedAt = long("updated_at")

	override val primaryKey = PrimaryKey(uuid, namespace, path)
}
