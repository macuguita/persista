package com.macuguita.persista.data

data class Identifier(
	val namespace: String,
	val path: String,
) : Comparable<Identifier> {
	init {
		require(isValidNamespace(namespace)) { "Invalid namespace '$namespace' in identifier: $this" }
		require(isValidPath(path)) { "Invalid path '$path' in identifier: $this" }
	}

	override fun toString() = "$namespace:$path"

	override fun compareTo(other: Identifier): Int {
		val result = path.compareTo(other.path)
		return if (result != 0) result else namespace.compareTo(other.namespace)
	}

	companion object {
		fun parse(identifier: String): Identifier {
			val i = identifier.indexOf(':')
			return when {
				i > 0 -> Identifier(identifier.substring(0, i), identifier.substring(i + 1))
				else -> throw IllegalArgumentException("Invalid identifier: $identifier")
			}
		}

		fun tryParse(identifier: String): Identifier? =
			try {
				parse(identifier)
			} catch (e: IllegalArgumentException) {
				null
			}

		fun isValidNamespace(namespace: String) =
			namespace != ".." && namespace.all { it == '_' || it == '-' || it in 'a'..'z' || it in '0'..'9' || it == '.' }

		fun isValidPath(path: String) = path.all { it == '_' || it == '-' || it in 'a'..'z' || it in '0'..'9' || it == '/' || it == '.' }
	}
}
