package com.macuguita.persista.data

object Config {
	val port = System.getenv("PORT")?.toInt() ?: error("PORT env var is required")
	val dbUrl = System.getenv("DATABASE_URL") ?: error("DATABASE_URL env var is required")
	val dbUser = System.getenv("DB_USER") ?: error("DB_USER env var is required")
	val dbPass = System.getenv("DB_PASSWORD") ?: error("DB_PASSWORD env var is required")
	val jwtSecret = System.getenv("JWT_SECRET") ?: error("JWT_SECRET env var is required")
	val adminSecret = System.getenv("ADMIN_SECRET") ?: error("ADMIN_SECRET env var is required")
}
