#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Helper functions
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

prompt_user() {
    local prompt="$1"
    local default="${2:-n}"

    while true; do
        echo -n -e "${CYAN}[PROMPT]${NC} $prompt (y/n): "
        read -r response
        response=${response:-$default}
        case $response in
            [Yy]* ) return 0;;
            [Nn]* ) return 1;;
            * ) echo "Please answer y or n.";;
        esac
    done
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        print_error "Please install Docker and try again"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running"
        print_error "Please start Docker and try again"
        exit 1
    fi

    print_success "Docker is available and running"
}

# Check if image exists
check_image_exists() {
    local image_name="$1"
    docker image inspect "$image_name" &>/dev/null
}

# Build MCP image
build_mcp_image() {
    print_status "Building MCP client image..."

    if docker build -t searxng-mcp .; then
        print_success "MCP client image built successfully"
        return 0
    else
        print_error "Failed to build MCP client image"
        return 1
    fi
}

# Check if container is currently running
check_running_container() {
    local container_name="$1"
    docker ps --format '{{.Names}}' | grep -q "^${container_name}$"
}

# Build SearXNG image
build_searxng_image() {
    print_status "Building SearXNG server image..."

    if ! cd docker &> /dev/null; then
        print_error "docker directory not found"
        return 1
    fi

    if docker build -t searxng-server .; then
        print_success "SearXNG server image built successfully"
        cd .. &> /dev/null
        return 0
    else
        print_error "Failed to build SearXNG server image"
        cd .. &> /dev/null
        return 1
    fi
}

# Start SearXNG container
start_searxng_container() {
    print_status "Starting SearXNG server container..."

    # Create network if it doesn't exist
    docker network create searxng 2>/dev/null || true

    # Check for running container first
    if check_running_container "searxng-server"; then
        print_warning "searxng-server container is currently running"
        if prompt_user "Would you like to stop the running container?" "y"; then
            print_status "Stopping and removing running container..."
            docker stop searxng-server 2>/dev/null || true
            docker rm searxng-server 2>/dev/null || true
        else
            print_warning "Cannot start new container while one is running"
            print_status "Use menu option 3 to stop the container first"
            return 1
        fi
    fi

    # Check for and remove stopped containers
    if docker ps -a --format 'table {{.Names}}' | grep -q "^searxng-server$"; then
        print_warning "Stopped searxng-server container found"
        print_status "Removing existing stopped container..."
        docker rm searxng-server 2>/dev/null || true
    fi

    # Run the Docker container
    if docker run -d --name searxng-server \
        --network searxng \
        -v "searxng_config:/etc/searxng/" \
        -v "searxng_data:/var/cache/searxng/" \
        -p 8080:8080 \
        searxng-server; then
        print_success "SearXNG server container started successfully"
    else
        print_error "Failed to start SearXNG server container"
        return 1
    fi
}

# Show current status
show_status() {
    echo -e "${BOLD}${CYAN}=== CURRENT STATUS ===${NC}"
    echo

    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "[Docker]: Not installed"
        return
    elif ! docker info &> /dev/null; then
        print_error "[Docker]: Not running"
        return
    else
        print_success "[Docker]: Running"
    fi

    # Check images
    if check_image_exists "searxng-server"; then
        print_success "[SearXNG Server Image]: found"
    else
        print_warning "[SearXNG Server Image]: not found"
    fi

    # Check container
    if check_running_container "searxng-server"; then
        print_success "[SearXNG Server Container]: running"
        echo -e "   ${GREEN}URL: http://localhost:8080${NC}"
    elif docker ps -a --format '{{.Names}}' | grep -q "^searxng-server$"; then
        print_warning "[SearXNG Server Container]: stopped"
    else
        print_warning "[SearXNG Server Container]: not found"
    fi

    if check_image_exists "searxng-mcp"; then
        print_success "[SearXNG MCP Image]: found"
    else
        print_warning "[SearXNG MCP Image]: not found"
    fi

    echo
}

# Show menu options
show_menu() {
    echo -e "${BOLD}${CYAN}=== MENU ===${NC}"
    echo "1. Build/Rebuild SearXNG image"
    echo "2. Remove        SearXNG image"
    echo "3. Start         SearXNG container"
    echo "4. Stop          SearXNG container"
    echo "5. Remove        SearXNG container"
    echo "6. Show          SearXNG container logs"
    echo "7. Build/Rebuild MCP image"
    echo "8. Remove        MCP image"
    echo "9. Exit"
    echo
    echo -n -e "${CYAN}Enter your choice (1-9):${NC} "
}

# Handle user choice
handle_choice() {
    local choice="$1"

    case $choice in
        1)
            echo
            if build_searxng_image; then
                print_success "SearXNG image build completed"
            else
                print_warning "SearXNG image build failed"
            fi
            ;;
        2)
            echo
            if check_image_exists "searxng-server"; then
                print_status "Removing searxng-server image..."
                docker rmi searxng-server -f
                print_success "Image removed"
            else
                print_warning "Image not found"
            fi
            ;;
        3)
            echo
            if start_searxng_container; then
                print_success "SearXNG container operation completed"
            else
                print_warning "SearXNG container operation failed or was cancelled"
            fi
            ;;
        4)
            echo
            if check_running_container "searxng-server"; then
                print_status "Stopping searxng-server container..."
                docker stop searxng-server
                print_success "Container stopped"
            else
                print_warning "Container is not running"
            fi
            ;;
        5)
            echo
            if docker ps -a --format '{{.Names}}' | grep -q "^searxng-server$"; then
                print_status "Removing searxng-server container..."
                docker stop searxng-server 2>/dev/null || true
                docker rm searxng-server
                print_success "Container removed"
            else
                print_warning "Container not found"
            fi
            ;;
        6)
            echo
            if check_running_container "searxng-server"; then
                print_status "Showing SearXNG container logs (Ctrl+C to exit):"
                docker logs -f searxng-server
            else
                print_warning "Container is not running"
            fi
            ;;
        7)
            echo
            if build_mcp_image; then
                print_success "MCP image build completed"
            else
                print_warning "MCP image build failed"
            fi
            ;;
        8)
            echo
            if check_image_exists "searxng-mcp"; then
                print_status "Removing searxng-mcp image..."
                docker rmi searxng-mcp -f
                print_success "MCP image removed"
            else
                print_warning "MCP image not found"
            fi
            ;;
        9)
            print_success "Goodbye!"
            exit 0
            ;;
        *)
            print_error "Invalid choice. Please enter 1-9."
            ;;
    esac
}



# Show header with ASCII art
show_header() {
    echo -e "${CYAN}${BOLD}"
    cat << 'EOF'
    ███████╗███████╗ █████╗ ██████╗ ██╗  ██╗███╗   ██╗ ██████╗
    ██╔════╝██╔════╝██╔══██╗██╔══██╗╚██╗██╔╝████╗  ██║██╔════╝
    ███████╗█████╗  ███████║██████╔╝ ╚███╔╝ ██╔██╗ ██║██║  ███╗
    ╚════██║██╔══╝  ██╔══██║██╔══██╗ ██╔██╗ ██║╚██╗██║██║   ██║
    ███████║███████╗██║  ██║██║  ██║██╔╝ ██╗██║ ╚████║╚██████╔╝
    ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝

                    ███╗   ███╗ ██████╗██████╗
                    ████╗ ████║██╔════╝██╔══██╗
                    ██╔████╔██║██║     ██████╔╝
                    ██║╚██╔╝██║██║     ██╔═══╝
                    ██║ ╚═╝ ██║╚██████╗██║
                    ╚═╝     ╚═╝ ╚═════╝╚═╝

                  🔍 Web search YOU control 🔍
EOF
    echo -e "${NC}"
    echo
}

# Main execution
main() {
    # Main menu loop
    while true; do
        clear
        show_header
        show_status
        show_menu
        read -r choice
        echo
        handle_choice "$choice"
        echo
        echo -e "${CYAN}Press Enter to continue...${NC}"
        read -r
    done
}

# Run main function
main "$@"
