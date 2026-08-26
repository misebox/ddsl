CREATE TABLE users (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  email text NOT NULL,
  name text NOT NULL,
  CONSTRAINT users_pkey PRIMARY KEY (id)
);

CREATE TABLE categories (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  note text NOT NULL,
  name text NOT NULL,
  CONSTRAINT categories_pkey PRIMARY KEY (id)
);

CREATE TABLE products (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  name text NOT NULL,
  price integer NOT NULL,
  CONSTRAINT products_pkey PRIMARY KEY (id)
);

CREATE TABLE orders (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  user_id integer NOT NULL,
  CONSTRAINT orders_pkey PRIMARY KEY (id)
);

CREATE TABLE order_items (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  order_id integer NOT NULL,
  product_id integer NOT NULL,
  quantity integer NOT NULL,
  CONSTRAINT order_items_pkey PRIMARY KEY (id)
);

CREATE TABLE posts (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  status text NOT NULL,
  published_at timestamptz,
  user_id integer NOT NULL,
  category_id integer NOT NULL,
  title text NOT NULL,
  CONSTRAINT posts_pkey PRIMARY KEY (id)
);

CREATE TABLE profiles (
  id serial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  user_id integer NOT NULL,
  bio text,
  CONSTRAINT profiles_pkey PRIMARY KEY (id)
);

CREATE TABLE post_histories (
  id serial NOT NULL,
  post_id integer NOT NULL,
  user_id integer NOT NULL,
  version integer NOT NULL,
  snapshot jsonb NOT NULL,
  recorded_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT post_histories_pkey PRIMARY KEY (id)
);

CREATE TABLE favorites (
  user_id integer NOT NULL,
  post_id integer NOT NULL,
  CONSTRAINT favorites_pkey PRIMARY KEY (user_id, post_id)
);

CREATE UNIQUE INDEX uq_users_email ON users (email);
CREATE UNIQUE INDEX uq_categories_name ON categories (name);
CREATE INDEX idx_orders_user_id ON orders (user_id);
CREATE INDEX idx_order_items_order_id ON order_items (order_id);
CREATE INDEX idx_order_items_product_id ON order_items (product_id);
CREATE INDEX idx_posts_status ON posts (status);
CREATE UNIQUE INDEX uq_posts_status_created_at ON posts (status, created_at);
CREATE INDEX idx_posts_user_id ON posts (user_id);
CREATE INDEX idx_posts_category_id ON posts (category_id);
CREATE UNIQUE INDEX uq_profiles_user_id ON profiles (user_id);
CREATE INDEX idx_post_histories_post_id ON post_histories (post_id);
CREATE INDEX idx_post_histories_user_id ON post_histories (user_id);

ALTER TABLE orders ADD CONSTRAINT orders_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE order_items ADD CONSTRAINT order_items_order_id_fkey FOREIGN KEY (order_id) REFERENCES orders (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE order_items ADD CONSTRAINT order_items_product_id_fkey FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE posts ADD CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE posts ADD CONSTRAINT posts_category_id_fkey FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE profiles ADD CONSTRAINT profiles_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE post_histories ADD CONSTRAINT post_histories_post_id_fkey FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE post_histories ADD CONSTRAINT post_histories_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE favorites ADD CONSTRAINT favorites_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE favorites ADD CONSTRAINT favorites_post_id_fkey FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE ON UPDATE CASCADE;

CREATE OR REPLACE FUNCTION users_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER users_updated_at_on_update_trg BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION users_updated_at_on_update();
CREATE OR REPLACE FUNCTION products_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER products_updated_at_on_update_trg BEFORE UPDATE ON products FOR EACH ROW EXECUTE FUNCTION products_updated_at_on_update();
CREATE OR REPLACE FUNCTION orders_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER orders_updated_at_on_update_trg BEFORE UPDATE ON orders FOR EACH ROW EXECUTE FUNCTION orders_updated_at_on_update();
CREATE OR REPLACE FUNCTION order_items_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER order_items_updated_at_on_update_trg BEFORE UPDATE ON order_items FOR EACH ROW EXECUTE FUNCTION order_items_updated_at_on_update();
CREATE OR REPLACE FUNCTION posts_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER posts_updated_at_on_update_trg BEFORE UPDATE ON posts FOR EACH ROW EXECUTE FUNCTION posts_updated_at_on_update();
CREATE OR REPLACE FUNCTION profiles_updated_at_on_update() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER profiles_updated_at_on_update_trg BEFORE UPDATE ON profiles FOR EACH ROW EXECUTE FUNCTION profiles_updated_at_on_update();

COMMENT ON TABLE users IS 'A person who signs in';
COMMENT ON COLUMN users.id IS 'Primary key';
COMMENT ON COLUMN users.created_at IS 'When the row was created';
COMMENT ON COLUMN users.updated_at IS 'When it last changed';
COMMENT ON COLUMN users.email IS 'Used to sign in';
COMMENT ON COLUMN users.name IS 'Shown to other users';
COMMENT ON TABLE categories IS 'A grouping for products';
COMMENT ON COLUMN categories.id IS 'Primary key';
COMMENT ON COLUMN categories.created_at IS 'When the row was created';
COMMENT ON COLUMN categories.note IS 'Anything worth noting';
COMMENT ON COLUMN categories.name IS 'Shown in navigation';
COMMENT ON TABLE products IS 'Something for sale';
COMMENT ON COLUMN products.id IS 'Primary key';
COMMENT ON COLUMN products.created_at IS 'When the row was created';
COMMENT ON COLUMN products.updated_at IS 'When it last changed';
COMMENT ON COLUMN products.name IS 'Shown to shoppers';
COMMENT ON COLUMN products.price IS 'Excluding tax, in yen';
COMMENT ON TABLE orders IS 'A purchase a customer placed';
COMMENT ON COLUMN orders.id IS 'Primary key';
COMMENT ON COLUMN orders.created_at IS 'When the row was created';
COMMENT ON COLUMN orders.updated_at IS 'When it last changed';
COMMENT ON COLUMN orders.user_id IS 'Who placed it';
COMMENT ON TABLE order_items IS 'One line of an order';
COMMENT ON COLUMN order_items.id IS 'Primary key';
COMMENT ON COLUMN order_items.created_at IS 'When the row was created';
COMMENT ON COLUMN order_items.updated_at IS 'When it last changed';
COMMENT ON COLUMN order_items.order_id IS 'Which order';
COMMENT ON COLUMN order_items.product_id IS 'What was bought';
COMMENT ON COLUMN order_items.quantity IS 'How many';
COMMENT ON TABLE posts IS 'Something a user wrote';
COMMENT ON COLUMN posts.id IS 'Primary key';
COMMENT ON COLUMN posts.created_at IS 'When the row was created';
COMMENT ON COLUMN posts.updated_at IS 'When it last changed';
COMMENT ON COLUMN posts.status IS 'Draft, scheduled or published';
COMMENT ON COLUMN posts.published_at IS 'When it went public';
COMMENT ON COLUMN posts.user_id IS 'Who wrote it';
COMMENT ON COLUMN posts.category_id IS 'Where it is filed';
COMMENT ON COLUMN posts.title IS 'Shown in listings';
COMMENT ON TABLE profiles IS 'Extra details about one user';
COMMENT ON COLUMN profiles.id IS 'Primary key';
COMMENT ON COLUMN profiles.created_at IS 'When the row was created';
COMMENT ON COLUMN profiles.updated_at IS 'When it last changed';
COMMENT ON COLUMN profiles.user_id IS 'Whose profile this is';
COMMENT ON COLUMN profiles.bio IS 'Free text the user wrote';
COMMENT ON TABLE post_histories IS 'Past states of a post';
COMMENT ON COLUMN post_histories.id IS 'Primary key';
COMMENT ON COLUMN post_histories.post_id IS 'What changed';
COMMENT ON COLUMN post_histories.user_id IS 'Who changed it';
COMMENT ON COLUMN post_histories.version IS 'Which revision';
COMMENT ON COLUMN post_histories.snapshot IS 'The row as it stood';
COMMENT ON COLUMN post_histories.recorded_at IS 'When it was recorded';
COMMENT ON TABLE favorites IS 'A user liking a post';
