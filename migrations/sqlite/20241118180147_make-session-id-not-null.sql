-- Add migration script here
CREATE TABLE "session_new" (
    "id" TEXT NOT NULL,
    "user_id" TEXT NOT NULL,
    "expires" INTEGER NOT NULL,
    "created" INTEGER NOT NULL,
    PRIMARY KEY("id"),
    FOREIGN KEY("user_id") REFERENCES "user"("name")
);

INSERT INTO "session_new" ("id", "user_id", "expires", "created")
SELECT "id", "user_id", "expires", "created" FROM "session";

DROP TABLE "session";

ALTER TABLE "session_new" RENAME TO "session";