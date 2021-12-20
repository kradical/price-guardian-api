package main

import "github.com/gofiber/fiber/v2"

func main() {
	app := fiber.New()

	api := app.Group("/api")

	api.Get("/", func(c *fiber.Ctx) error {
		return c.SendString("")
	})
	api.Get("/health", func(c *fiber.Ctx) error {
		return c.SendString("")
	})

	app.Listen(":8000")
}
