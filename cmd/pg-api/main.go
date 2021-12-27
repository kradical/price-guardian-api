package main

import (
	"context"
	"crypto/ed25519"
	"encoding/base64"
	"errors"
	"log"
	"os"
	"reflect"
	"strconv"
	"strings"
	"time"

	"github.com/andskur/argon2-hashing"
	"github.com/go-playground/validator/v10"
	"github.com/gofiber/fiber/v2"
	jwtware "github.com/gofiber/jwt/v3"
	"github.com/golang-jwt/jwt/v4"
	"github.com/google/uuid"
	"github.com/jackc/pgconn"
	"github.com/jackc/pgx/v4"
	"github.com/jackc/pgx/v4/pgxpool"
	_ "github.com/joho/godotenv/autoload"
)

type NewUser struct {
	Email    string `json:"email" validate:"required,email"`
	Password string `json:"password" validate:"required,min=8"`
}
type User struct {
	Id    int    `json:"id"`
	Email string `json:"email"`
}

// use a single instance of Validate, it caches struct info
var validate *validator.Validate

func msgForTag(fe validator.FieldError) string {
	switch fe.Tag() {
	case "required":
		return "This field is required"
	case "email":
		return "Invalid email"
	case "min":
		return "Must be at least length " + fe.Param()
	}
	return fe.Error() // default error
}

func main() {
	jwtKeyBytes, err := base64.StdEncoding.DecodeString(os.Getenv("JWT_PRIVATE_KEY"))
	if err != nil {
		log.Fatalf("Unable to parse jwt private key: %v\n", err)
	}

	jwtPrivateKey := ed25519.PrivateKey(jwtKeyBytes)
	jwtPublicKey := jwtPrivateKey.Public()

	dbPool, err := pgxpool.Connect(context.Background(), os.Getenv("DATABASE_URL"))
	if err != nil {
		log.Fatalf("Unable to connect to database: %v\n", err)
	}
	defer dbPool.Close()

	validate = validator.New()
	validate.RegisterTagNameFunc(func(fld reflect.StructField) string {
		name := strings.SplitN(fld.Tag.Get("json"), ",", 2)[0]

		if name == "-" {
			return ""
		}

		return name
	})

	app := fiber.New()

	api := app.Group("/api")

	api.Get("/health", func(c *fiber.Ctx) error {
		return c.Send(nil)
	})
	api.Post("/auth/tokens", func(c *fiber.Ctx) error {
		loginUser := new(NewUser)

		if err := c.BodyParser(loginUser); err != nil {
			return err
		}

		sqlStatement := "SELECT id, password FROM users WHERE email = $1"

		var userId int
		var hash string
		err = dbPool.QueryRow(context.Background(), sqlStatement, loginUser.Email).Scan(&userId, &hash)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
					"email": "Incorrect email address",
				})
			}

			return err
		}

		err = argon2.CompareHashAndPassword([]byte(hash), []byte(loginUser.Password))
		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"password": "Incorrect password",
			})
		}

		claims := jwt.StandardClaims{
			ExpiresAt: time.Now().Unix() + 30*24*60*60,
			Id:        uuid.New().String(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    c.BaseURL(),
			Subject:   strconv.Itoa(userId),
		}

		token := jwt.NewWithClaims(&jwt.SigningMethodEd25519{}, claims)

		tokenString, err := token.SignedString(ed25519.PrivateKey(jwtPrivateKey))
		if err != nil {
			return err
		}

		return c.JSON(fiber.Map{
			"token": tokenString,
		})
	})
	api.Post("/users", func(c *fiber.Ctx) error {
		newUser := new(NewUser)

		if err := c.BodyParser(newUser); err != nil {
			return err
		}

		err := validate.Struct(newUser)
		if err != nil {
			var errors fiber.Map
			for _, err := range err.(validator.ValidationErrors) {
				errors[err.Field()] = msgForTag(err)
			}

			return c.Status(fiber.ErrBadRequest.Code).JSON(errors)
		}

		hash, err := argon2.GenerateFromPassword([]byte(newUser.Password), argon2.DefaultParams)
		if err != nil {
			return err
		}

		sqlStatement := `
INSERT INTO users (email, password)
VALUES ($1, $2)
RETURNING id`
		var id int
		err = dbPool.QueryRow(context.Background(), sqlStatement, newUser.Email, hash).Scan(&id)

		if err != nil {
			var pgErr *pgconn.PgError
			if errors.As(err, &pgErr) && pgErr.ConstraintName == "users_email_key" {
				msg := "Email " + newUser.Email + " is already in use."
				return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
					"email": msg,
				})
			}

			return err
		}

		return c.JSON(User{id, newUser.Email})
	})

	// All Unauthenticated routes above here
	api.Use(jwtware.New(jwtware.Config{
		SigningKey:    jwtPublicKey,
		SigningMethod: "EdDSA",
		ContextKey:    "token",
		Claims:        &jwt.StandardClaims{},
	}))
	api.Use(func(c *fiber.Ctx) error {
		token := c.Locals("token").(*jwt.Token)
		claims := token.Claims.(*jwt.StandardClaims)
		userId, _ := strconv.Atoi(claims.Subject)
		c.Locals("userId", userId)

		return c.Next()
	})

	api.Get("/users/me", func(c *fiber.Ctx) error {
		userId := c.Locals("userId").(int)

		sqlStatement := `
SELECT email
FROM users
WHERE id = $1`
		user := User{userId, ""}
		err = dbPool.QueryRow(context.Background(), sqlStatement, userId).Scan(&user.Email)

		return c.JSON(user)
	})

	log.Fatal(app.Listen(":8000"))
}
