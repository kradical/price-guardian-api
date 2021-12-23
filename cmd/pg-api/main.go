package main

import (
	"context"
	"errors"
	"log"
	"os"
	"reflect"
	"strings"

	"github.com/go-playground/validator/v10"
	"github.com/gofiber/fiber/v2"
	"github.com/jackc/pgconn"
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

		// TODO: Hash Password

		sqlStatement := `
INSERT INTO users (email, password)
VALUES ($1, $2)
RETURNING id`
		var id int
		err = dbPool.QueryRow(context.Background(), sqlStatement, newUser.Email, newUser.Password).Scan(&id)

		if err != nil {
			var pgErr *pgconn.PgError
			if errors.As(err, &pgErr) && pgErr.ConstraintName == "users_email_key" {
				msg := "Email " + newUser.Email + " is already in use."
				return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
					"email": msg,
				})
			}

			return fiber.ErrInternalServerError
		}

		return c.JSON(User{id, newUser.Email})
	})

	log.Fatal(app.Listen(":8000"))
}
