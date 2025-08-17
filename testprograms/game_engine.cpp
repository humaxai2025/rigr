/**
 * Comprehensive Game Engine System in C++
 * 
 * This program demonstrates advanced C++ programming concepts including:
 * - Object-oriented design with inheritance and polymorphism
 * - Template programming and generic algorithms
 * - STL containers and algorithms
 * - Smart pointers and modern memory management
 * - Exception handling and RAII
 * - Design patterns (Factory, Observer, Strategy)
 * - Multi-threading and concurrency
 * - Lambda expressions and functional programming
 * - Move semantics and perfect forwarding
 * - Operator overloading
 * - Virtual functions and abstract classes
 */

#include <iostream>
#include <vector>
#include <memory>
#include <string>
#include <unordered_map>
#include <algorithm>
#include <random>
#include <chrono>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <future>
#include <functional>
#include <queue>
#include <fstream>
#include <sstream>
#include <typeinfo>
#include <cassert>
#include <cmath>

// Forward declarations
class GameObject;
class Component;
class Transform;
class Renderer;
class Physics;
class AudioSource;
class GameEngine;

// Utility classes and templates
template<typename T>
class Singleton {
public:
    static T& GetInstance() {
        static T instance;
        return instance;
    }
    
protected:
    Singleton() = default;
    virtual ~Singleton() = default;
    Singleton(const Singleton&) = delete;
    Singleton& operator=(const Singleton&) = delete;
};

template<typename T>
struct Vector3D {
    T x, y, z;
    
    Vector3D() : x(0), y(0), z(0) {}
    Vector3D(T x, T y, T z) : x(x), y(y), z(z) {}
    
    Vector3D operator+(const Vector3D& other) const {
        return Vector3D(x + other.x, y + other.y, z + other.z);
    }
    
    Vector3D operator-(const Vector3D& other) const {
        return Vector3D(x - other.x, y - other.y, z - other.z);
    }
    
    Vector3D operator*(T scalar) const {
        return Vector3D(x * scalar, y * scalar, z * scalar);
    }
    
    T dot(const Vector3D& other) const {
        return x * other.x + y * other.y + z * other.z;
    }
    
    T magnitude() const {
        return std::sqrt(x * x + y * y + z * z);
    }
    
    Vector3D normalized() const {
        T mag = magnitude();
        if (mag > 0) {
            return *this * (1.0f / mag);
        }
        return Vector3D();
    }
};

using Vector3f = Vector3D<float>;
using Vector3i = Vector3D<int>;

// Exception classes
class GameEngineException : public std::runtime_error {
public:
    explicit GameEngineException(const std::string& message) 
        : std::runtime_error("Game Engine Error: " + message) {}
};

class ResourceNotFoundException : public GameEngineException {
public:
    explicit ResourceNotFoundException(const std::string& resource) 
        : GameEngineException("Resource not found: " + resource) {}
};

class ComponentNotFoundException : public GameEngineException {
public:
    explicit ComponentNotFoundException(const std::string& component) 
        : GameEngineException("Component not found: " + component) {}
};

// Component System
class Component {
public:
    virtual ~Component() = default;
    virtual void Update(float deltaTime) {}
    virtual void Initialize() {}
    virtual void Destroy() {}
    virtual std::string GetType() const = 0;
    
    bool IsEnabled() const { return enabled_; }
    void SetEnabled(bool enabled) { enabled_ = enabled; }
    
    std::weak_ptr<GameObject> GetGameObject() const { return gameObject_; }
    void SetGameObject(std::weak_ptr<GameObject> gameObject) { gameObject_ = gameObject; }
    
private:
    bool enabled_ = true;
    std::weak_ptr<GameObject> gameObject_;
};

// Transform Component
class Transform : public Component {
private:
    Vector3f position_;
    Vector3f rotation_;
    Vector3f scale_;
    std::weak_ptr<Transform> parent_;
    std::vector<std::weak_ptr<Transform>> children_;
    
public:
    Transform() : position_(0, 0, 0), rotation_(0, 0, 0), scale_(1, 1, 1) {}
    
    std::string GetType() const override { return "Transform"; }
    
    // Position
    const Vector3f& GetPosition() const { return position_; }
    void SetPosition(const Vector3f& position) { position_ = position; }
    void Translate(const Vector3f& translation) { position_ = position_ + translation; }
    
    // Rotation
    const Vector3f& GetRotation() const { return rotation_; }
    void SetRotation(const Vector3f& rotation) { rotation_ = rotation; }
    void Rotate(const Vector3f& rotation) { rotation_ = rotation_ + rotation; }
    
    // Scale
    const Vector3f& GetScale() const { return scale_; }
    void SetScale(const Vector3f& scale) { scale_ = scale; }
    
    // Hierarchy
    void SetParent(std::weak_ptr<Transform> parent) {
        if (auto oldParent = parent_.lock()) {
            auto& children = oldParent->children_;
            children.erase(std::remove_if(children.begin(), children.end(),
                [this](const std::weak_ptr<Transform>& child) {
                    return child.expired() || child.lock().get() == this;
                }), children.end());
        }
        
        parent_ = parent;
        if (auto newParent = parent.lock()) {
            newParent->children_.push_back(shared_from_this());
        }
    }
    
    Vector3f GetWorldPosition() const {
        if (auto parentTransform = parent_.lock()) {
            return parentTransform->GetWorldPosition() + position_;
        }
        return position_;
    }
    
    std::shared_ptr<Transform> shared_from_this() {
        // Note: This is simplified. In real code, you'd inherit from std::enable_shared_from_this
        return std::static_pointer_cast<Transform>(GetGameObject().lock()->GetComponent<Transform>());
    }
};

// Renderer Component
class Renderer : public Component {
private:
    std::string meshName_;
    std::string materialName_;
    bool visible_;
    
public:
    Renderer(const std::string& mesh = "default_cube", const std::string& material = "default_material") 
        : meshName_(mesh), materialName_(material), visible_(true) {}
    
    std::string GetType() const override { return "Renderer"; }
    
    void Update(float deltaTime) override {
        if (visible_ && IsEnabled()) {
            // Simulate rendering
            static int frameCount = 0;
            if (++frameCount % 60 == 0) {
                std::cout << "Rendering " << meshName_ << " with " << materialName_ << std::endl;
            }
        }
    }
    
    const std::string& GetMesh() const { return meshName_; }
    void SetMesh(const std::string& mesh) { meshName_ = mesh; }
    
    const std::string& GetMaterial() const { return materialName_; }
    void SetMaterial(const std::string& material) { materialName_ = material; }
    
    bool IsVisible() const { return visible_; }
    void SetVisible(bool visible) { visible_ = visible; }
};

// Physics Component
class Physics : public Component {
private:
    Vector3f velocity_;
    Vector3f acceleration_;
    float mass_;
    bool useGravity_;
    bool isStatic_;
    
public:
    Physics(float mass = 1.0f, bool useGravity = true) 
        : velocity_(0, 0, 0), acceleration_(0, 0, 0), mass_(mass), useGravity_(useGravity), isStatic_(false) {}
    
    std::string GetType() const override { return "Physics"; }
    
    void Update(float deltaTime) override {
        if (isStatic_ || !IsEnabled()) return;
        
        auto gameObject = GetGameObject().lock();
        if (!gameObject) return;
        
        auto transform = gameObject->GetComponent<Transform>();
        if (!transform) return;
        
        // Apply gravity
        if (useGravity_) {
            acceleration_ = acceleration_ + Vector3f(0, -9.81f, 0);
        }
        
        // Update velocity and position
        velocity_ = velocity_ + acceleration_ * deltaTime;
        transform->Translate(velocity_ * deltaTime);
        
        // Reset acceleration
        acceleration_ = Vector3f(0, 0, 0);
    }
    
    void AddForce(const Vector3f& force) {
        if (!isStatic_ && mass_ > 0) {
            acceleration_ = acceleration_ + force * (1.0f / mass_);
        }
    }
    
    void SetVelocity(const Vector3f& velocity) { velocity_ = velocity; }
    const Vector3f& GetVelocity() const { return velocity_; }
    
    float GetMass() const { return mass_; }
    void SetMass(float mass) { mass_ = std::max(0.1f, mass); }
    
    bool GetUseGravity() const { return useGravity_; }
    void SetUseGravity(bool useGravity) { useGravity_ = useGravity; }
    
    bool IsStatic() const { return isStatic_; }
    void SetStatic(bool isStatic) { isStatic_ = isStatic; }
};

// Audio Source Component
class AudioSource : public Component {
private:
    std::string audioClip_;
    float volume_;
    bool looping_;
    bool playing_;
    
public:
    AudioSource(const std::string& clip = "", float volume = 1.0f) 
        : audioClip_(clip), volume_(volume), looping_(false), playing_(false) {}
    
    std::string GetType() const override { return "AudioSource"; }
    
    void Play() {
        if (!audioClip_.empty() && IsEnabled()) {
            playing_ = true;
            std::cout << "Playing audio: " << audioClip_ << " at volume " << volume_ << std::endl;
        }
    }
    
    void Stop() {
        playing_ = false;
        std::cout << "Stopped audio: " << audioClip_ << std::endl;
    }
    
    void SetClip(const std::string& clip) { audioClip_ = clip; }
    const std::string& GetClip() const { return audioClip_; }
    
    void SetVolume(float volume) { volume_ = std::clamp(volume, 0.0f, 1.0f); }
    float GetVolume() const { return volume_; }
    
    void SetLooping(bool looping) { looping_ = looping; }
    bool IsLooping() const { return looping_; }
    
    bool IsPlaying() const { return playing_; }
};

// GameObject class
class GameObject : public std::enable_shared_from_this<GameObject> {
private:
    std::string name_;
    std::vector<std::unique_ptr<Component>> components_;
    bool active_;
    
public:
    GameObject(const std::string& name = "GameObject") : name_(name), active_(true) {
        // Every GameObject has a Transform component
        AddComponent<Transform>();
    }
    
    virtual ~GameObject() = default;
    
    template<typename T, typename... Args>
    std::shared_ptr<T> AddComponent(Args&&... args) {
        static_assert(std::is_base_of_v<Component, T>, "T must be derived from Component");
        
        auto component = std::make_unique<T>(std::forward<Args>(args)...);
        auto componentPtr = component.get();
        component->SetGameObject(weak_from_this());
        component->Initialize();
        components_.push_back(std::move(component));
        
        return std::shared_ptr<T>(shared_from_this(), componentPtr);
    }
    
    template<typename T>
    std::shared_ptr<T> GetComponent() const {
        static_assert(std::is_base_of_v<Component, T>, "T must be derived from Component");
        
        for (const auto& component : components_) {
            if (auto casted = dynamic_cast<T*>(component.get())) {
                return std::shared_ptr<T>(shared_from_this(), casted);
            }
        }
        return nullptr;
    }
    
    template<typename T>
    void RemoveComponent() {
        static_assert(std::is_base_of_v<Component, T>, "T must be derived from Component");
        
        components_.erase(
            std::remove_if(components_.begin(), components_.end(),
                [](const std::unique_ptr<Component>& component) {
                    return dynamic_cast<T*>(component.get()) != nullptr;
                }), components_.end());
    }
    
    void Update(float deltaTime) {
        if (!active_) return;
        
        for (auto& component : components_) {
            if (component && component->IsEnabled()) {
                component->Update(deltaTime);
            }
        }
    }
    
    void Destroy() {
        for (auto& component : components_) {
            if (component) {
                component->Destroy();
            }
        }
        components_.clear();
    }
    
    const std::string& GetName() const { return name_; }
    void SetName(const std::string& name) { name_ = name; }
    
    bool IsActive() const { return active_; }
    void SetActive(bool active) { active_ = active; }
    
    std::vector<std::string> GetComponentTypes() const {
        std::vector<std::string> types;
        for (const auto& component : components_) {
            if (component) {
                types.push_back(component->GetType());
            }
        }
        return types;
    }
};

// Event System
template<typename... Args>
class Event {
private:
    std::vector<std::function<void(Args...)>> listeners_;
    mutable std::mutex mutex_;
    
public:
    void Subscribe(const std::function<void(Args...)>& listener) {
        std::lock_guard<std::mutex> lock(mutex_);
        listeners_.push_back(listener);
    }
    
    void Unsubscribe(const std::function<void(Args...)>& listener) {
        std::lock_guard<std::mutex> lock(mutex_);
        // Note: This is simplified. In real code, you'd need a way to identify listeners
        listeners_.clear();
    }
    
    void Invoke(Args... args) const {
        std::lock_guard<std::mutex> lock(mutex_);
        for (const auto& listener : listeners_) {
            listener(args...);
        }
    }
};

// Resource Manager
template<typename T>
class ResourceManager {
private:
    std::unordered_map<std::string, std::shared_ptr<T>> resources_;
    mutable std::shared_mutex mutex_;
    
public:
    void Load(const std::string& name, std::shared_ptr<T> resource) {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        resources_[name] = resource;
    }
    
    std::shared_ptr<T> Get(const std::string& name) const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        auto it = resources_.find(name);
        if (it != resources_.end()) {
            return it->second;
        }
        throw ResourceNotFoundException(name);
    }
    
    bool Exists(const std::string& name) const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        return resources_.find(name) != resources_.end();
    }
    
    void Unload(const std::string& name) {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        resources_.erase(name);
    }
    
    void Clear() {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        resources_.clear();
    }
    
    std::vector<std::string> GetLoadedResourceNames() const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        std::vector<std::string> names;
        for (const auto& [name, resource] : resources_) {
            names.push_back(name);
        }
        return names;
    }
};

// Scene Management
class Scene {
private:
    std::string name_;
    std::vector<std::shared_ptr<GameObject>> gameObjects_;
    mutable std::shared_mutex mutex_;
    
public:
    Scene(const std::string& name) : name_(name) {}
    
    void AddGameObject(std::shared_ptr<GameObject> gameObject) {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        gameObjects_.push_back(gameObject);
    }
    
    void RemoveGameObject(std::shared_ptr<GameObject> gameObject) {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        gameObjects_.erase(
            std::remove(gameObjects_.begin(), gameObjects_.end(), gameObject),
            gameObjects_.end());
    }
    
    void Update(float deltaTime) {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        for (auto& gameObject : gameObjects_) {
            if (gameObject && gameObject->IsActive()) {
                gameObject->Update(deltaTime);
            }
        }
    }
    
    void Clear() {
        std::unique_lock<std::shared_mutex> lock(mutex_);
        for (auto& gameObject : gameObjects_) {
            if (gameObject) {
                gameObject->Destroy();
            }
        }
        gameObjects_.clear();
    }
    
    template<typename T>
    std::vector<std::shared_ptr<GameObject>> FindObjectsWithComponent() const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        std::vector<std::shared_ptr<GameObject>> result;
        
        std::copy_if(gameObjects_.begin(), gameObjects_.end(), std::back_inserter(result),
            [](const std::shared_ptr<GameObject>& obj) {
                return obj && obj->GetComponent<T>() != nullptr;
            });
        
        return result;
    }
    
    std::shared_ptr<GameObject> FindObjectByName(const std::string& name) const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        auto it = std::find_if(gameObjects_.begin(), gameObjects_.end(),
            [&name](const std::shared_ptr<GameObject>& obj) {
                return obj && obj->GetName() == name;
            });
        
        return (it != gameObjects_.end()) ? *it : nullptr;
    }
    
    const std::string& GetName() const { return name_; }
    size_t GetObjectCount() const {
        std::shared_lock<std::shared_mutex> lock(mutex_);
        return gameObjects_.size();
    }
};

// Input System
enum class KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Alpha0, Alpha1, Alpha2, Alpha3, Alpha4, Alpha5, Alpha6, Alpha7, Alpha8, Alpha9,
    Space, Enter, Escape, LeftArrow, RightArrow, UpArrow, DownArrow
};

class Input : public Singleton<Input> {
private:
    std::unordered_map<KeyCode, bool> keyStates_;
    std::unordered_map<KeyCode, bool> previousKeyStates_;
    Vector3f mousePosition_;
    mutable std::mutex mutex_;
    
public:
    void Update() {
        std::lock_guard<std::mutex> lock(mutex_);
        previousKeyStates_ = keyStates_;
        
        // Simulate random key presses for demo
        static std::random_device rd;
        static std::mt19937 gen(rd());
        static std::uniform_real_distribution<> dis(0.0, 1.0);
        
        if (dis(gen) < 0.01) { // 1% chance per frame
            KeyCode randomKey = static_cast<KeyCode>(static_cast<int>(KeyCode::A) + (gen() % 26));
            keyStates_[randomKey] = !keyStates_[randomKey];
        }
    }
    
    bool IsKeyDown(KeyCode key) const {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = keyStates_.find(key);
        return it != keyStates_.end() && it->second;
    }
    
    bool IsKeyPressed(KeyCode key) const {
        std::lock_guard<std::mutex> lock(mutex_);
        auto current = keyStates_.find(key);
        auto previous = previousKeyStates_.find(key);
        
        bool currentState = current != keyStates_.end() && current->second;
        bool previousState = previous != previousKeyStates_.end() && previous->second;
        
        return currentState && !previousState;
    }
    
    bool IsKeyReleased(KeyCode key) const {
        std::lock_guard<std::mutex> lock(mutex_);
        auto current = keyStates_.find(key);
        auto previous = previousKeyStates_.find(key);
        
        bool currentState = current != keyStates_.end() && current->second;
        bool previousState = previous != previousKeyStates_.end() && previous->second;
        
        return !currentState && previousState;
    }
    
    const Vector3f& GetMousePosition() const { return mousePosition_; }
    void SetMousePosition(const Vector3f& position) {
        std::lock_guard<std::mutex> lock(mutex_);
        mousePosition_ = position;
    }
};

// Game Engine
class GameEngine : public Singleton<GameEngine> {
private:
    std::unique_ptr<Scene> currentScene_;
    ResourceManager<std::string> textureManager_;
    ResourceManager<std::string> audioManager_;
    ResourceManager<std::string> modelManager_;
    
    bool running_;
    float targetFrameRate_;
    std::chrono::high_resolution_clock::time_point lastFrameTime_;
    float deltaTime_;
    int frameCount_;
    float elapsedTime_;
    
    Event<float> onUpdate_;
    Event<> onFixedUpdate_;
    
    std::thread gameThread_;
    std::mutex engineMutex_;
    
    // Thread pool for async operations
    class ThreadPool {
    private:
        std::vector<std::thread> workers_;
        std::queue<std::function<void()>> tasks_;
        std::mutex queueMutex_;
        std::condition_variable condition_;
        bool stop_;
        
    public:
        ThreadPool(size_t numThreads) : stop_(false) {
            for (size_t i = 0; i < numThreads; ++i) {
                workers_.emplace_back([this] {
                    while (true) {
                        std::function<void()> task;
                        {
                            std::unique_lock<std::mutex> lock(queueMutex_);
                            condition_.wait(lock, [this] { return stop_ || !tasks_.empty(); });
                            
                            if (stop_ && tasks_.empty()) return;
                            
                            task = std::move(tasks_.front());
                            tasks_.pop();
                        }
                        task();
                    }
                });
            }
        }
        
        ~ThreadPool() {
            {
                std::unique_lock<std::mutex> lock(queueMutex_);
                stop_ = true;
            }
            condition_.notify_all();
            for (std::thread& worker : workers_) {
                worker.join();
            }
        }
        
        template<typename F, typename... Args>
        auto enqueue(F&& f, Args&&... args) -> std::future<typename std::result_of<F(Args...)>::type> {
            using return_type = typename std::result_of<F(Args...)>::type;
            
            auto task = std::make_shared<std::packaged_task<return_type()>>(
                std::bind(std::forward<F>(f), std::forward<Args>(args)...)
            );
            
            std::future<return_type> res = task->get_future();
            {
                std::unique_lock<std::mutex> lock(queueMutex_);
                if (stop_) {
                    throw std::runtime_error("enqueue on stopped ThreadPool");
                }
                tasks_.emplace([task]() { (*task)(); });
            }
            condition_.notify_one();
            return res;
        }
    };
    
    std::unique_ptr<ThreadPool> threadPool_;
    
public:
    GameEngine() : running_(false), targetFrameRate_(60.0f), deltaTime_(0.0f), 
                   frameCount_(0), elapsedTime_(0.0f), threadPool_(std::make_unique<ThreadPool>(4)) {
        currentScene_ = std::make_unique<Scene>("DefaultScene");
        
        // Load default resources
        textureManager_.Load("default", std::make_shared<std::string>("default_texture"));
        audioManager_.Load("default", std::make_shared<std::string>("default_audio"));
        modelManager_.Load("default_cube", std::make_shared<std::string>("default_cube_model"));
    }
    
    void Start() {
        std::lock_guard<std::mutex> lock(engineMutex_);
        if (running_) return;
        
        running_ = true;
        lastFrameTime_ = std::chrono::high_resolution_clock::now();
        
        std::cout << "Game Engine Started!" << std::endl;
        std::cout << "Target FPS: " << targetFrameRate_ << std::endl;
        
        gameThread_ = std::thread(&GameEngine::GameLoop, this);
    }
    
    void Stop() {
        {
            std::lock_guard<std::mutex> lock(engineMutex_);
            running_ = false;
        }
        
        if (gameThread_.joinable()) {
            gameThread_.join();
        }
        
        std::cout << "Game Engine Stopped!" << std::endl;
    }
    
    void LoadScene(std::unique_ptr<Scene> scene) {
        std::lock_guard<std::mutex> lock(engineMutex_);
        if (currentScene_) {
            currentScene_->Clear();
        }
        currentScene_ = std::move(scene);
    }
    
    Scene* GetCurrentScene() const {
        std::lock_guard<std::mutex> lock(engineMutex_);
        return currentScene_.get();
    }
    
    template<typename T>
    ResourceManager<T>& GetResourceManager() {
        static_assert(std::is_same_v<T, std::string>, "Only string resources supported in this demo");
        return textureManager_; // Simplified for demo
    }
    
    float GetDeltaTime() const { return deltaTime_; }
    int GetFrameCount() const { return frameCount_; }
    float GetElapsedTime() const { return elapsedTime_; }
    float GetFPS() const { return 1.0f / deltaTime_; }
    
    void SetTargetFrameRate(float fps) {
        std::lock_guard<std::mutex> lock(engineMutex_);
        targetFrameRate_ = fps;
    }
    
    Event<float>& OnUpdate() { return onUpdate_; }
    Event<>& OnFixedUpdate() { return onFixedUpdate_; }
    
    template<typename F, typename... Args>
    auto ExecuteAsync(F&& f, Args&&... args) {
        return threadPool_->enqueue(std::forward<F>(f), std::forward<Args>(args)...);
    }
    
private:
    void GameLoop() {
        const auto targetFrameDuration = std::chrono::duration<float>(1.0f / targetFrameRate_);
        auto fixedUpdateAccumulator = std::chrono::duration<float>(0);
        const auto fixedDeltaTime = std::chrono::duration<float>(1.0f / 50.0f); // 50 FPS fixed update
        
        while (running_) {
            auto currentTime = std::chrono::high_resolution_clock::now();
            auto frameDuration = std::chrono::duration<float>(currentTime - lastFrameTime_);
            lastFrameTime_ = currentTime;
            
            deltaTime_ = frameDuration.count();
            elapsedTime_ += deltaTime_;
            frameCount_++;
            
            // Update input
            Input::GetInstance().Update();
            
            // Update current scene
            if (currentScene_) {
                currentScene_->Update(deltaTime_);
            }
            
            // Invoke update events
            onUpdate_.Invoke(deltaTime_);
            
            // Fixed update loop
            fixedUpdateAccumulator += frameDuration;
            while (fixedUpdateAccumulator >= fixedDeltaTime) {
                onFixedUpdate_.Invoke();
                fixedUpdateAccumulator -= fixedDeltaTime;
            }
            
            // Frame rate limiting
            auto frameEnd = std::chrono::high_resolution_clock::now();
            auto frameTime = frameEnd - currentTime;
            
            if (frameTime < targetFrameDuration) {
                std::this_thread::sleep_for(targetFrameDuration - frameTime);
            }
        }
    }
    
    mutable std::mutex engineMutex_;
};

// Custom Component Examples
class PlayerController : public Component {
private:
    float moveSpeed_;
    float jumpForce_;
    bool isGrounded_;
    
public:
    PlayerController(float moveSpeed = 5.0f, float jumpForce = 10.0f) 
        : moveSpeed_(moveSpeed), jumpForce_(jumpForce), isGrounded_(true) {}
    
    std::string GetType() const override { return "PlayerController"; }
    
    void Update(float deltaTime) override {
        if (!IsEnabled()) return;
        
        auto gameObject = GetGameObject().lock();
        if (!gameObject) return;
        
        auto transform = gameObject->GetComponent<Transform>();
        auto physics = gameObject->GetComponent<Physics>();
        
        if (!transform || !physics) return;
        
        Input& input = Input::GetInstance();
        Vector3f movement(0, 0, 0);
        
        // Movement input
        if (input.IsKeyDown(KeyCode::A) || input.IsKeyDown(KeyCode::LeftArrow)) {
            movement.x -= moveSpeed_ * deltaTime;
        }
        if (input.IsKeyDown(KeyCode::D) || input.IsKeyDown(KeyCode::RightArrow)) {
            movement.x += moveSpeed_ * deltaTime;
        }
        if (input.IsKeyDown(KeyCode::W) || input.IsKeyDown(KeyCode::UpArrow)) {
            movement.z += moveSpeed_ * deltaTime;
        }
        if (input.IsKeyDown(KeyCode::S) || input.IsKeyDown(KeyCode::DownArrow)) {
            movement.z -= moveSpeed_ * deltaTime;
        }
        
        // Jump input
        if (input.IsKeyPressed(KeyCode::Space) && isGrounded_) {
            physics->AddForce(Vector3f(0, jumpForce_, 0));
            isGrounded_ = false;
        }
        
        // Apply movement
        if (movement.magnitude() > 0) {
            transform->Translate(movement);
            
            // Simple ground check
            if (transform->GetPosition().y <= 0) {
                auto pos = transform->GetPosition();
                pos.y = 0;
                transform->SetPosition(pos);
                isGrounded_ = true;
            }
        }
    }
    
    float GetMoveSpeed() const { return moveSpeed_; }
    void SetMoveSpeed(float speed) { moveSpeed_ = speed; }
    
    float GetJumpForce() const { return jumpForce_; }
    void SetJumpForce(float force) { jumpForce_ = force; }
};

class RotatingComponent : public Component {
private:
    Vector3f rotationSpeed_;
    
public:
    RotatingComponent(const Vector3f& rotationSpeed = Vector3f(0, 90, 0)) 
        : rotationSpeed_(rotationSpeed) {}
    
    std::string GetType() const override { return "RotatingComponent"; }
    
    void Update(float deltaTime) override {
        if (!IsEnabled()) return;
        
        auto gameObject = GetGameObject().lock();
        if (!gameObject) return;
        
        auto transform = gameObject->GetComponent<Transform>();
        if (!transform) return;
        
        Vector3f rotation = rotationSpeed_ * deltaTime;
        transform->Rotate(rotation);
    }
    
    const Vector3f& GetRotationSpeed() const { return rotationSpeed_; }
    void SetRotationSpeed(const Vector3f& speed) { rotationSpeed_ = speed; }
};

// Factory Pattern for GameObject creation
class GameObjectFactory {
public:
    static std::shared_ptr<GameObject> CreatePlayer(const std::string& name = "Player") {
        auto player = std::make_shared<GameObject>(name);
        
        player->GetComponent<Transform>()->SetPosition(Vector3f(0, 0, 0));
        player->AddComponent<Renderer>("player_mesh", "player_material");
        player->AddComponent<Physics>(75.0f, true); // 75kg player with gravity
        player->AddComponent<PlayerController>(8.0f, 12.0f);
        player->AddComponent<AudioSource>("player_sounds", 0.8f);
        
        return player;
    }
    
    static std::shared_ptr<GameObject> CreateEnemy(const std::string& name = "Enemy") {
        auto enemy = std::make_shared<GameObject>(name);
        
        enemy->GetComponent<Transform>()->SetPosition(Vector3f(5, 0, 0));
        enemy->AddComponent<Renderer>("enemy_mesh", "enemy_material");
        enemy->AddComponent<Physics>(60.0f, true);
        enemy->AddComponent<RotatingComponent>(Vector3f(0, 45, 0));
        enemy->AddComponent<AudioSource>("enemy_sounds", 0.6f);
        
        return enemy;
    }
    
    static std::shared_ptr<GameObject> CreatePickup(const std::string& name = "Pickup") {
        auto pickup = std::make_shared<GameObject>(name);
        
        pickup->GetComponent<Transform>()->SetPosition(Vector3f(-3, 1, 2));
        pickup->GetComponent<Transform>()->SetScale(Vector3f(0.5f, 0.5f, 0.5f));
        pickup->AddComponent<Renderer>("pickup_mesh", "pickup_material");
        pickup->AddComponent<RotatingComponent>(Vector3f(0, 180, 0));
        pickup->AddComponent<AudioSource>("pickup_sound", 1.0f);
        
        return pickup;
    }
};

// Demo and testing functions
void RunComponentTests() {
    std::cout << "\n=== Component System Tests ===" << std::endl;
    
    // Test GameObject creation and component management
    auto testObject = std::make_shared<GameObject>("TestObject");
    
    // Test component addition
    auto renderer = testObject->AddComponent<Renderer>("test_mesh", "test_material");
    auto physics = testObject->AddComponent<Physics>(10.0f, false);
    auto audio = testObject->AddComponent<AudioSource>("test_audio", 0.7f);
    
    std::cout << "Created test object with components:" << std::endl;
    auto componentTypes = testObject->GetComponentTypes();
    for (const auto& type : componentTypes) {
        std::cout << "  - " << type << std::endl;
    }
    
    // Test component retrieval and manipulation
    if (auto retrievedPhysics = testObject->GetComponent<Physics>()) {
        retrievedPhysics->AddForce(Vector3f(10, 0, 0));
        std::cout << "Applied force to physics component" << std::endl;
    }
    
    // Test component removal
    testObject->RemoveComponent<AudioSource>();
    std::cout << "Removed AudioSource component" << std::endl;
    
    componentTypes = testObject->GetComponentTypes();
    std::cout << "Remaining components:" << std::endl;
    for (const auto& type : componentTypes) {
        std::cout << "  - " << type << std::endl;
    }
}

void RunSceneTests() {
    std::cout << "\n=== Scene Management Tests ===" << std::endl;
    
    auto testScene = std::make_unique<Scene>("TestScene");
    
    // Create and add game objects
    auto player = GameObjectFactory::CreatePlayer("TestPlayer");
    auto enemy = GameObjectFactory::CreateEnemy("TestEnemy");
    auto pickup = GameObjectFactory::CreatePickup("TestPickup");
    
    testScene->AddGameObject(player);
    testScene->AddGameObject(enemy);
    testScene->AddGameObject(pickup);
    
    std::cout << "Created test scene with " << testScene->GetObjectCount() << " objects" << std::endl;
    
    // Test object finding
    auto foundPlayer = testScene->FindObjectByName("TestPlayer");
    if (foundPlayer) {
        std::cout << "Found player object: " << foundPlayer->GetName() << std::endl;
    }
    
    // Test component-based object finding
    auto physicsObjects = testScene->FindObjectsWithComponent<Physics>();
    std::cout << "Found " << physicsObjects.size() << " objects with Physics components" << std::endl;
    
    // Test scene update
    testScene->Update(1.0f / 60.0f); // 60 FPS delta time
    std::cout << "Updated scene for one frame" << std::endl;
}

void RunResourceManagerTests() {
    std::cout << "\n=== Resource Manager Tests ===" << std::endl;
    
    ResourceManager<std::string> resourceManager;
    
    // Load resources
    resourceManager.Load("texture1", std::make_shared<std::string>("grass_texture.png"));
    resourceManager.Load("texture2", std::make_shared<std::string>("rock_texture.png"));
    resourceManager.Load("audio1", std::make_shared<std::string>("background_music.wav"));
    
    std::cout << "Loaded test resources" << std::endl;
    
    // Test resource retrieval
    try {
        auto texture = resourceManager.Get("texture1");
        std::cout << "Retrieved resource: " << *texture << std::endl;
    } catch (const ResourceNotFoundException& e) {
        std::cout << "Resource error: " << e.what() << std::endl;
    }
    
    // Test resource existence
    std::cout << "texture1 exists: " << (resourceManager.Exists("texture1") ? "Yes" : "No") << std::endl;
    std::cout << "nonexistent exists: " << (resourceManager.Exists("nonexistent") ? "Yes" : "No") << std::endl;
    
    // List loaded resources
    auto loadedResources = resourceManager.GetLoadedResourceNames();
    std::cout << "Loaded resources:" << std::endl;
    for (const auto& name : loadedResources) {
        std::cout << "  - " << name << std::endl;
    }
}

void RunPerformanceTests() {
    std::cout << "\n=== Performance Tests ===" << std::endl;
    
    auto perfScene = std::make_unique<Scene>("PerformanceTestScene");
    
    // Create many game objects
    const int numObjects = 1000;
    auto start = std::chrono::high_resolution_clock::now();
    
    for (int i = 0; i < numObjects; ++i) {
        auto obj = std::make_shared<GameObject>("PerfObject" + std::to_string(i));
        obj->GetComponent<Transform>()->SetPosition(Vector3f(
            static_cast<float>(i % 100), 
            0, 
            static_cast<float>(i / 100)
        ));
        obj->AddComponent<Renderer>("default_mesh", "default_material");
        obj->AddComponent<Physics>(1.0f, false);
        perfScene->AddGameObject(obj);
    }
    
    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    
    std::cout << "Created " << numObjects << " objects in " << duration.count() << "ms" << std::endl;
    
    // Test update performance
    start = std::chrono::high_resolution_clock::now();
    const int numUpdates = 100;
    
    for (int i = 0; i < numUpdates; ++i) {
        perfScene->Update(1.0f / 60.0f);
    }
    
    end = std::chrono::high_resolution_clock::now();
    duration = std::chrono::duration_cast<std::chrono::microseconds>(end - start);
    
    std::cout << "Updated " << numObjects << " objects " << numUpdates 
              << " times in " << duration.count() << "μs" << std::endl;
    std::cout << "Average update time per frame: " << (duration.count() / numUpdates) << "μs" << std::endl;
}

void RunAsyncTests() {
    std::cout << "\n=== Async Operations Tests ===" << std::endl;
    
    GameEngine& engine = GameEngine::GetInstance();
    
    // Test async resource loading simulation
    std::vector<std::future<std::string>> futures;
    
    for (int i = 0; i < 5; ++i) {
        futures.push_back(engine.ExecuteAsync([i]() -> std::string {
            std::this_thread::sleep_for(std::chrono::milliseconds(100 + (i * 50)));
            return "AsyncResource" + std::to_string(i) + " loaded";
        }));
    }
    
    std::cout << "Started " << futures.size() << " async operations" << std::endl;
    
    for (auto& future : futures) {
        try {
            auto result = future.get();
            std::cout << result << std::endl;
        } catch (const std::exception& e) {
            std::cout << "Async operation failed: " << e.what() << std::endl;
        }
    }
}

// Main demonstration program
int main() {
    try {
        std::cout << "=== Comprehensive C++ Game Engine System ===" << std::endl;
        std::cout << "Initializing game engine..." << std::endl;
        
        GameEngine& engine = GameEngine::GetInstance();
        
        // Set up event handlers
        engine.OnUpdate().Subscribe([](float deltaTime) {
            static int updateCount = 0;
            if (++updateCount % 300 == 0) { // Every 5 seconds at 60 FPS
                std::cout << "Engine Update - Delta: " << deltaTime 
                          << "s, FPS: " << (1.0f / deltaTime) << std::endl;
            }
        });
        
        engine.OnFixedUpdate().Subscribe([]() {
            static int fixedUpdateCount = 0;
            if (++fixedUpdateCount % 250 == 0) { // Every 5 seconds at 50 FPS
                std::cout << "Fixed Update #" << fixedUpdateCount << std::endl;
            }
        });
        
        // Run tests
        RunComponentTests();
        RunSceneTests();
        RunResourceManagerTests();
        RunAsyncTests();
        RunPerformanceTests();
        
        // Create game scene
        auto gameScene = std::make_unique<Scene>("GameScene");
        
        // Create game objects using factory
        auto player = GameObjectFactory::CreatePlayer("MainPlayer");
        auto enemy1 = GameObjectFactory::CreateEnemy("Enemy1");
        auto enemy2 = GameObjectFactory::CreateEnemy("Enemy2");
        auto pickup1 = GameObjectFactory::CreatePickup("HealthPickup");
        auto pickup2 = GameObjectFactory::CreatePickup("AmmoPickup");
        
        // Position objects
        enemy2->GetComponent<Transform>()->SetPosition(Vector3f(-5, 0, 3));
        pickup2->GetComponent<Transform>()->SetPosition(Vector3f(0, 1, -3));
        
        // Add objects to scene
        gameScene->AddGameObject(player);
        gameScene->AddGameObject(enemy1);
        gameScene->AddGameObject(enemy2);
        gameScene->AddGameObject(pickup1);
        gameScene->AddGameObject(pickup2);
        
        // Load the scene
        engine.LoadScene(std::move(gameScene));
        
        // Start the engine
        engine.Start();
        
        // Let the engine run for a demo period
        std::cout << "\nRunning game engine demo for 10 seconds..." << std::endl;
        std::this_thread::sleep_for(std::chrono::seconds(10));
        
        // Demonstrate runtime object manipulation
        std::cout << "\nDemonstrating runtime object manipulation..." << std::endl;
        auto currentScene = engine.GetCurrentScene();
        if (currentScene) {
            auto foundPlayer = currentScene->FindObjectByName("MainPlayer");
            if (foundPlayer) {
                // Modify player properties
                if (auto playerController = foundPlayer->GetComponent<PlayerController>()) {
                    playerController->SetMoveSpeed(15.0f);
                    std::cout << "Increased player move speed to 15.0" << std::endl;
                }
                
                if (auto playerPhysics = foundPlayer->GetComponent<Physics>()) {
                    playerPhysics->AddForce(Vector3f(0, 20, 0));
                    std::cout << "Applied upward force to player" << std::endl;
                }
            }
            
            // Test component-based queries
            auto allPhysicsObjects = currentScene->FindObjectsWithComponent<Physics>();
            std::cout << "Found " << allPhysicsObjects.size() << " objects with physics" << std::endl;
            
            auto allRenderableObjects = currentScene->FindObjectsWithComponent<Renderer>();
            std::cout << "Found " << allRenderableObjects.size() << " renderable objects" << std::endl;
        }
        
        // Continue running for a bit more
        std::this_thread::sleep_for(std::chrono::seconds(5));
        
        // Stop the engine
        engine.Stop();
        
        // Display final statistics
        std::cout << "\n=== Final Engine Statistics ===" << std::endl;
        std::cout << "Total frames rendered: " << engine.GetFrameCount() << std::endl;
        std::cout << "Total elapsed time: " << engine.GetElapsedTime() << "s" << std::endl;
        std::cout << "Average FPS: " << (engine.GetFrameCount() / engine.GetElapsedTime()) << std::endl;
        
        std::cout << "\nGame Engine demonstration completed successfully!" << std::endl;
        std::cout << "\nKey C++ features demonstrated:" << std::endl;
        std::cout << "- Object-oriented programming with inheritance and polymorphism" << std::endl;
        std::cout << "- Template programming and generic algorithms" << std::endl;
        std::cout << "- STL containers and algorithms" << std::endl;
        std::cout << "- Smart pointers and modern memory management" << std::endl;
        std::cout << "- Exception handling and RAII" << std::endl;
        std::cout << "- Design patterns (Singleton, Factory, Observer)" << std::endl;
        std::cout << "- Multi-threading and concurrency" << std::endl;
        std::cout << "- Lambda expressions and functional programming" << std::endl;
        std::cout << "- Move semantics and perfect forwarding" << std::endl;
        std::cout << "- Operator overloading" << std::endl;
        std::cout << "- Virtual functions and abstract classes" << std::endl;
        
    } catch (const std::exception& e) {
        std::cerr << "Fatal error: " << e.what() << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "Unknown fatal error occurred" << std::endl;
        return 1;
    }
    
    return 0;
}